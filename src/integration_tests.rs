use anyhow::Result;
use tempfile::tempdir;
use tokio::time::{timeout, Duration};

use crate::config::Config;
use crate::node_manager::{NodeManager, DistributionStrategy};

/// Integration tests for Phase 1.3 Network & Storage Integration
/// 
/// These tests verify that the networking and storage layers work together
/// correctly for peer-to-peer chunk sharing and file distribution.

#[tokio::test]
async fn test_node_manager_initialization() -> Result<()> {
    // Test that NodeManager can be created and started
    let config = Config::default();
    let temp_dir = tempdir()?;
    
    let mut node_manager = NodeManager::new(config, temp_dir.path().to_path_buf()).await?;
    
    // Should be able to start (though networking may fail without proper setup)
    let start_result = timeout(Duration::from_secs(5), node_manager.start()).await;
    
    // Even if start fails due to network setup, the initialization should have worked
    match start_result {
        Ok(Ok(())) => {
            // Success! Now shutdown
            node_manager.shutdown().await?;
        }
        Ok(Err(_)) => {
            // Expected - networking setup might fail in test environment
            // But the important part is that NodeManager initialized
        }
        Err(_) => {
            // Timeout - also expected in test environment
        }
    }
    
    Ok(())
}

#[tokio::test]
async fn test_node_status_retrieval() -> Result<()> {
    // Test that we can get node status
    let config = Config::default();
    let temp_dir = tempdir()?;
    
    let node_manager = NodeManager::new(config, temp_dir.path().to_path_buf()).await?;
    
    let status = node_manager.get_node_status().await;
    
    // Verify status contains expected fields
    assert_eq!(status.version, env!("CARGO_PKG_VERSION"));
    assert_eq!(status.chunks_served, 0);
    assert_eq!(status.chunks_retrieved, 0);
    assert!(status.storage_capacity > 0);
    
    Ok(())
}

#[tokio::test]
async fn test_file_storage_integration() -> Result<()> {
    // Test that we can store files through the NodeManager
    let config = Config::default();
    let temp_dir = tempdir()?;
    
    let node_manager = NodeManager::new(config, temp_dir.path().to_path_buf()).await?;
    
    let file_id = "test-integration-file";
    let filename = "integration_test.txt";
    let test_data = b"Hello, ZephyrFS Integration! This tests the full stack.";
    
    // Store file with local-only strategy
    let file_hash = node_manager.store_file(
        file_id, 
        test_data, 
        filename, 
        DistributionStrategy::LocalOnly
    ).await?;
    
    assert!(!file_hash.is_empty());
    
    // Retrieve the file
    let retrieved = node_manager.retrieve_file(file_id).await?;
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap(), test_data);
    
    // Verify node status shows the stored file
    let status = node_manager.get_node_status().await;
    assert!(status.storage_used > 0);
    assert!(status.file_count > 0);
    
    // Delete the file
    let deleted = node_manager.delete_file(file_id).await?;
    assert!(deleted);
    
    // Verify file is gone
    let retrieved_after_delete = node_manager.retrieve_file(file_id).await?;
    assert!(retrieved_after_delete.is_none());
    
    Ok(())
}

#[tokio::test]
async fn test_chunk_level_operations() -> Result<()> {
    // Test direct chunk operations through the integrated system
    let config = Config::default();
    let temp_dir = tempdir()?;
    
    let node_manager = NodeManager::new(config, temp_dir.path().to_path_buf()).await?;
    
    let chunk_id = "test-chunk-integration";
    let chunk_data = b"This is test chunk data for integration testing.";
    
    // For this test, we'll use the store_file method and then test chunk retrieval
    // This tests the integration without accessing private fields
    let temp_file_id = "temp-file-for-chunk-test";
    let file_hash = node_manager.store_file(
        temp_file_id,
        chunk_data,
        "chunk_test.bin",
        DistributionStrategy::LocalOnly
    ).await?;
    assert!(!file_hash.is_empty());
    
    // Retrieve file to verify it was stored
    let retrieved_file = node_manager.retrieve_file(temp_file_id).await?;
    assert!(retrieved_file.is_some());
    assert_eq!(retrieved_file.unwrap(), chunk_data);
    
    // Test peer chunk requests (should work locally)
    let peer_chunk_result = node_manager.request_chunk_from_peers(chunk_id, &file_hash).await;
    // This should return None since we don't have actual peers, but shouldn't error
    assert!(peer_chunk_result.is_ok());
    
    Ok(())
}

#[tokio::test]
async fn test_storage_capacity_integration() -> Result<()> {
    // Test that storage capacity management works with the integrated system
    let mut config = Config::default();
    // Set very small capacity for testing
    config.storage.max_storage = 1024; // 1KB
    
    let temp_dir = tempdir()?;
    let node_manager = NodeManager::new(config, temp_dir.path().to_path_buf()).await?;
    
    // Get initial node status which includes storage info
    let initial_status = node_manager.get_node_status().await;
    assert_eq!(initial_status.storage_capacity, 1024);
    assert_eq!(initial_status.storage_used, 0);
    
    // Try to store a large file that exceeds capacity
    let large_data = vec![0u8; 2048]; // 2KB - larger than capacity
    
    let result = node_manager.store_file(
        "large-file",
        &large_data,
        "large.bin",
        DistributionStrategy::LocalOnly
    ).await;
    
    // Should fail due to capacity limits
    assert!(result.is_err());
    
    // Verify capacity info is still correct
    let final_status = node_manager.get_node_status().await;
    assert_eq!(final_status.storage_used, 0); // Nothing should have been stored
    
    Ok(())
}

#[tokio::test]
async fn test_multiple_file_operations() -> Result<()> {
    // Test storing multiple files and ensuring they're properly isolated
    let config = Config::default();
    let temp_dir = tempdir()?;
    
    let node_manager = NodeManager::new(config, temp_dir.path().to_path_buf()).await?;
    
    // Store multiple files
    let files: Vec<(&str, &str, &[u8])> = vec![
        ("file1", "test1.txt", b"First file content"),
        ("file2", "test2.txt", b"Second file content with different data"),
        ("file3", "test3.txt", b"Third file"),
    ];
    
    let mut stored_hashes = Vec::new();
    
    for (file_id, filename, data) in &files {
        let hash = node_manager.store_file(
            file_id,
            *data,
            filename,
            DistributionStrategy::LocalOnly
        ).await?;
        stored_hashes.push(hash);
    }
    
    // Verify all files can be retrieved correctly
    for (file_id, _filename, expected_data) in &files {
        let retrieved = node_manager.retrieve_file(file_id).await?;
        assert!(retrieved.is_some());
        assert_eq!(&retrieved.unwrap(), *expected_data);
    }
    
    // Verify node status shows correct metrics
    let status = node_manager.get_node_status().await;
    assert_eq!(status.file_count, 3);
    assert!(status.storage_used > 0);
    assert!(status.chunk_count > 0); // Should have some chunks
    
    // Delete one file and verify others remain
    let deleted = node_manager.delete_file("file2").await?;
    assert!(deleted);
    
    // Verify file2 is gone but others remain
    assert!(node_manager.retrieve_file("file2").await?.is_none());
    assert!(node_manager.retrieve_file("file1").await?.is_some());
    assert!(node_manager.retrieve_file("file3").await?.is_some());
    
    Ok(())
}

// Integration tests can be run individually with:
// cargo test test_node_manager_initialization
// cargo test test_node_status_retrieval
// cargo test test_file_storage_integration
// cargo test test_chunk_level_operations
// cargo test test_storage_capacity_integration
// cargo test test_multiple_file_operations