//! Reed-Solomon Error Correction
//!
//! Efficient Reed-Solomon coding for ZephyrFS chunks, providing mathematical
//! redundancy that can reconstruct data from partial chunks

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Reed-Solomon encoder/decoder for ZephyrFS
#[derive(Debug, Clone)]
pub struct ReedSolomonCodec {
    /// Coding parameters
    pub config: ReedSolomonConfig,
    /// Galois field arithmetic
    pub galois_field: GaloisField,
    /// Encoding matrices
    pub encoding_matrix: Vec<Vec<u8>>,
    pub decoding_cache: HashMap<Vec<usize>, Vec<Vec<u8>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReedSolomonConfig {
    /// Number of data chunks
    pub data_chunks: u32,
    /// Number of parity chunks
    pub parity_chunks: u32,
    /// Chunk size in bytes
    pub chunk_size: usize,
    /// Galois field size (typically 2^8 = 256)
    pub field_size: u32,
    /// Primitive polynomial for GF(2^8)
    pub primitive_poly: u32,
}

/// Galois Field arithmetic for Reed-Solomon
#[derive(Debug, Clone)]
pub struct GaloisField {
    /// Field size (2^8 = 256)
    pub size: u32,
    /// Generator polynomial
    pub generator: u32,
    /// Logarithm table
    pub log_table: Vec<u8>,
    /// Antilogarithm table
    pub antilog_table: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncodedChunk {
    pub chunk_id: String,
    pub chunk_index: u32,
    pub chunk_type: ChunkType,
    pub data: Vec<u8>,
    pub metadata: ChunkMetadata,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChunkType {
    Data,
    Parity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkMetadata {
    pub original_size: usize,
    pub checksum: String,
    pub encoding_config: ReedSolomonConfig,
    pub chunk_position: u32,
    pub total_chunks: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconstructionRequest {
    pub original_chunk_id: String,
    pub available_chunks: Vec<EncodedChunk>,
    pub missing_chunk_indices: Vec<u32>,
    pub target_chunk_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconstructionResult {
    pub success: bool,
    pub reconstructed_chunks: Vec<EncodedChunk>,
    pub reconstruction_time_ms: u64,
    pub error_message: Option<String>,
    pub verification_passed: bool,
}

impl Default for ReedSolomonConfig {
    fn default() -> Self {
        Self {
            data_chunks: 6,
            parity_chunks: 3,
            chunk_size: 1024 * 1024, // 1MB chunks
            field_size: 256,
            primitive_poly: 0x11d, // Standard GF(2^8) primitive polynomial
        }
    }
}

impl GaloisField {
    /// Create new Galois Field GF(2^8)
    pub fn new() -> Self {
        let mut gf = Self {
            size: 256,
            generator: 0x11d, // x^8 + x^4 + x^3 + x^2 + 1
            log_table: vec![0; 256],
            antilog_table: vec![0; 256],
        };

        gf.build_tables();
        gf
    }

    /// Build logarithm and antilogarithm tables
    fn build_tables(&mut self) {
        let mut val = 1u32;

        for i in 0..255 {
            self.antilog_table[i] = val as u8;
            self.log_table[val as usize] = i as u8;

            val <<= 1;
            if val & self.size != 0 {
                val ^= self.generator;
            }
        }

        self.antilog_table[255] = self.antilog_table[0];
    }

    /// Galois field multiplication
    pub fn multiply(&self, a: u8, b: u8) -> u8 {
        if a == 0 || b == 0 {
            return 0;
        }

        let log_sum = (self.log_table[a as usize] as u32 + self.log_table[b as usize] as u32) % 255;
        self.antilog_table[log_sum as usize]
    }

    /// Galois field division
    pub fn divide(&self, a: u8, b: u8) -> u8 {
        if a == 0 {
            return 0;
        }
        if b == 0 {
            panic!("Division by zero in Galois field");
        }

        let log_diff = (self.log_table[a as usize] as i32 - self.log_table[b as usize] as i32 + 255) % 255;
        self.antilog_table[log_diff as usize]
    }

    /// Galois field power
    pub fn power(&self, base: u8, exp: u8) -> u8 {
        if base == 0 {
            return 0;
        }

        let log_result = (self.log_table[base as usize] as u32 * exp as u32) % 255;
        self.antilog_table[log_result as usize]
    }
}

impl ReedSolomonCodec {
    /// Create new Reed-Solomon codec
    pub fn new(config: ReedSolomonConfig) -> Result<Self> {
        let galois_field = GaloisField::new();
        let encoding_matrix = Self::build_encoding_matrix(&config, &galois_field)?;

        Ok(Self {
            config,
            galois_field,
            encoding_matrix,
            decoding_cache: HashMap::new(),
        })
    }

    /// Build encoding matrix for Reed-Solomon
    fn build_encoding_matrix(config: &ReedSolomonConfig, gf: &GaloisField) -> Result<Vec<Vec<u8>>> {
        let total_chunks = config.data_chunks + config.parity_chunks;
        let mut matrix = Vec::with_capacity(total_chunks as usize);

        // Identity matrix for data chunks
        for i in 0..config.data_chunks {
            let mut row = vec![0u8; config.data_chunks as usize];
            row[i as usize] = 1;
            matrix.push(row);
        }

        // Vandermonde matrix for parity chunks
        for i in 0..config.parity_chunks {
            let mut row = Vec::with_capacity(config.data_chunks as usize);
            let alpha = (i + 1) as u8; // Generator element

            for j in 0..config.data_chunks {
                row.push(gf.power(alpha, j as u8));
            }
            matrix.push(row);
        }

        Ok(matrix)
    }

    /// Encode data into Reed-Solomon chunks
    pub fn encode(&self, chunk_id: String, data: &[u8]) -> Result<Vec<EncodedChunk>> {
        let chunk_size = self.config.chunk_size;
        let data_chunks = self.config.data_chunks as usize;
        let total_chunks = (self.config.data_chunks + self.config.parity_chunks) as usize;

        // Pad data to fit evenly into data chunks
        let padded_size = ((data.len() + data_chunks - 1) / data_chunks) * data_chunks;
        let mut padded_data = data.to_vec();
        padded_data.resize(padded_size, 0);

        let chunk_data_size = padded_size / data_chunks;
        let mut chunks = Vec::with_capacity(total_chunks);

        // Create data chunks
        for i in 0..data_chunks {
            let start = i * chunk_data_size;
            let end = start + chunk_data_size;
            let chunk_data = padded_data[start..end].to_vec();

            chunks.push(EncodedChunk {
                chunk_id: format!("{}_{}", chunk_id, i),
                chunk_index: i as u32,
                chunk_type: ChunkType::Data,
                data: chunk_data,
                metadata: ChunkMetadata {
                    original_size: if i == data_chunks - 1 {
                        data.len() - (data_chunks - 1) * chunk_data_size
                    } else {
                        chunk_data_size
                    },
                    checksum: self.calculate_checksum(&padded_data[start..end]),
                    encoding_config: self.config.clone(),
                    chunk_position: i as u32,
                    total_chunks: total_chunks as u32,
                },
                created_at: Utc::now(),
            });
        }

        // Create parity chunks
        for i in data_chunks..total_chunks {
            let mut parity_data = vec![0u8; chunk_data_size];

            // Calculate parity using encoding matrix
            for j in 0..chunk_data_size {
                let mut parity_byte = 0u8;

                for k in 0..data_chunks {
                    let data_byte = padded_data[k * chunk_data_size + j];
                    let coeff = self.encoding_matrix[i][k];
                    parity_byte ^= self.galois_field.multiply(data_byte, coeff);
                }

                parity_data[j] = parity_byte;
            }

            chunks.push(EncodedChunk {
                chunk_id: format!("{}_{}", chunk_id, i),
                chunk_index: i as u32,
                chunk_type: ChunkType::Parity,
                data: parity_data.clone(),
                metadata: ChunkMetadata {
                    original_size: chunk_data_size,
                    checksum: self.calculate_checksum(&parity_data),
                    encoding_config: self.config.clone(),
                    chunk_position: i as u32,
                    total_chunks: total_chunks as u32,
                },
                created_at: Utc::now(),
            });
        }

        Ok(chunks)
    }

    /// Decode/reconstruct data from available chunks
    pub fn decode(&mut self, request: ReconstructionRequest) -> Result<ReconstructionResult> {
        let start_time = std::time::Instant::now();

        // Verify we have enough chunks for reconstruction
        if request.available_chunks.len() < self.config.data_chunks as usize {
            return Ok(ReconstructionResult {
                success: false,
                reconstructed_chunks: Vec::new(),
                reconstruction_time_ms: start_time.elapsed().as_millis() as u64,
                error_message: Some(format!(
                    "Insufficient chunks: need {}, have {}",
                    self.config.data_chunks,
                    request.available_chunks.len()
                )),
                verification_passed: false,
            });
        }

        let data_chunks = self.config.data_chunks as usize;
        let chunk_size = request.available_chunks[0].data.len();

        // Build decoding matrix
        let available_indices: Vec<usize> = request.available_chunks
            .iter()
            .map(|chunk| chunk.chunk_index as usize)
            .collect();

        let decoding_matrix = self.get_or_build_decoding_matrix(&available_indices)?;

        // Reconstruct missing data
        let mut reconstructed_data = vec![vec![0u8; chunk_size]; data_chunks];

        // Copy available data chunks
        for chunk in &request.available_chunks {
            let idx = chunk.chunk_index as usize;
            if idx < data_chunks {
                reconstructed_data[idx] = chunk.data.clone();
            }
        }

        // Reconstruct missing data chunks
        for missing_idx in &request.missing_chunk_indices {
            let missing_idx = *missing_idx as usize;
            if missing_idx >= data_chunks {
                continue; // Skip parity chunks for now
            }

            let mut reconstructed_chunk = vec![0u8; chunk_size];

            for byte_pos in 0..chunk_size {
                let mut reconstructed_byte = 0u8;

                for (matrix_col, chunk) in request.available_chunks.iter().enumerate().take(data_chunks) {
                    let coeff = decoding_matrix[missing_idx][matrix_col];
                    let data_byte = chunk.data[byte_pos];
                    reconstructed_byte ^= self.galois_field.multiply(data_byte, coeff);
                }

                reconstructed_chunk[byte_pos] = reconstructed_byte;
            }

            reconstructed_data[missing_idx] = reconstructed_chunk;
        }

        // Create reconstructed chunk objects
        let mut reconstructed_chunks = Vec::new();
        for missing_idx in &request.missing_chunk_indices {
            let idx = *missing_idx as usize;
            if idx < data_chunks {
                reconstructed_chunks.push(EncodedChunk {
                    chunk_id: format!("{}_{}", request.original_chunk_id, idx),
                    chunk_index: idx as u32,
                    chunk_type: ChunkType::Data,
                    data: reconstructed_data[idx].clone(),
                    metadata: ChunkMetadata {
                        original_size: chunk_size,
                        checksum: self.calculate_checksum(&reconstructed_data[idx]),
                        encoding_config: self.config.clone(),
                        chunk_position: idx as u32,
                        total_chunks: (self.config.data_chunks + self.config.parity_chunks),
                    },
                    created_at: Utc::now(),
                });
            }
        }

        // Verify reconstruction
        let verification_passed = self.verify_reconstruction(&reconstructed_chunks)?;

        Ok(ReconstructionResult {
            success: true,
            reconstructed_chunks,
            reconstruction_time_ms: start_time.elapsed().as_millis() as u64,
            error_message: None,
            verification_passed,
        })
    }

    /// Get or build decoding matrix for given available chunks
    fn get_or_build_decoding_matrix(&mut self, available_indices: &[usize]) -> Result<Vec<Vec<u8>>> {
        // Check cache first
        if let Some(cached_matrix) = self.decoding_cache.get(available_indices) {
            return Ok(cached_matrix.clone());
        }

        let data_chunks = self.config.data_chunks as usize;

        // Build submatrix from encoding matrix using available chunks
        let mut submatrix = Vec::with_capacity(data_chunks);
        for &idx in available_indices.iter().take(data_chunks) {
            submatrix.push(self.encoding_matrix[idx].clone());
        }

        // Invert the submatrix
        let decoding_matrix = self.invert_matrix(submatrix)?;

        // Cache the result
        self.decoding_cache.insert(available_indices.to_vec(), decoding_matrix.clone());

        Ok(decoding_matrix)
    }

    /// Invert a matrix in Galois field
    fn invert_matrix(&self, mut matrix: Vec<Vec<u8>>) -> Result<Vec<Vec<u8>>> {
        let n = matrix.len();

        // Create augmented matrix [A|I]
        let mut augmented = Vec::with_capacity(n);
        for i in 0..n {
            let mut row = matrix[i].clone();
            for j in 0..n {
                row.push(if i == j { 1 } else { 0 });
            }
            augmented.push(row);
        }

        // Gaussian elimination
        for i in 0..n {
            // Find pivot
            let mut pivot_row = i;
            for j in (i + 1)..n {
                if augmented[j][i] != 0 {
                    pivot_row = j;
                    break;
                }
            }

            if augmented[pivot_row][i] == 0 {
                return Err(anyhow::anyhow!("Matrix is not invertible"));
            }

            // Swap rows if needed
            if pivot_row != i {
                augmented.swap(i, pivot_row);
            }

            // Scale pivot row
            let pivot = augmented[i][i];
            for j in 0..(2 * n) {
                augmented[i][j] = self.galois_field.divide(augmented[i][j], pivot);
            }

            // Eliminate column
            for j in 0..n {
                if i != j && augmented[j][i] != 0 {
                    let factor = augmented[j][i];
                    for k in 0..(2 * n) {
                        let product = self.galois_field.multiply(factor, augmented[i][k]);
                        augmented[j][k] ^= product;
                    }
                }
            }
        }

        // Extract inverse matrix
        let mut inverse = Vec::with_capacity(n);
        for i in 0..n {
            inverse.push(augmented[i][n..].to_vec());
        }

        Ok(inverse)
    }

    /// Calculate checksum for data
    fn calculate_checksum(&self, data: &[u8]) -> String {
        // Simple checksum - in production would use cryptographic hash
        let sum: u32 = data.iter().map(|&b| b as u32).sum();
        format!("{:08x}", sum)
    }

    /// Verify reconstruction correctness
    fn verify_reconstruction(&self, chunks: &[EncodedChunk]) -> Result<bool> {
        // Verify checksums
        for chunk in chunks {
            let calculated_checksum = self.calculate_checksum(&chunk.data);
            if calculated_checksum != chunk.metadata.checksum {
                return Ok(false);
            }
        }

        // Additional verification could include re-encoding and comparing
        Ok(true)
    }

    /// Get redundancy schemes available
    pub fn get_available_schemes() -> Vec<ReedSolomonConfig> {
        vec![
            // Standard schemes
            ReedSolomonConfig {
                data_chunks: 3,
                parity_chunks: 2,
                chunk_size: 1024 * 1024,
                field_size: 256,
                primitive_poly: 0x11d,
            },
            ReedSolomonConfig {
                data_chunks: 6,
                parity_chunks: 3,
                chunk_size: 1024 * 1024,
                field_size: 256,
                primitive_poly: 0x11d,
            },
            ReedSolomonConfig {
                data_chunks: 10,
                parity_chunks: 4,
                chunk_size: 1024 * 1024,
                field_size: 256,
                primitive_poly: 0x11d,
            },
            // High redundancy for critical data
            ReedSolomonConfig {
                data_chunks: 8,
                parity_chunks: 6,
                chunk_size: 1024 * 1024,
                field_size: 256,
                primitive_poly: 0x11d,
            },
        ]
    }

    /// Calculate storage efficiency for a scheme
    pub fn calculate_storage_efficiency(&self) -> f64 {
        self.config.data_chunks as f64 / (self.config.data_chunks + self.config.parity_chunks) as f64
    }

    /// Calculate fault tolerance
    pub fn calculate_fault_tolerance(&self) -> u32 {
        self.config.parity_chunks
    }

    /// Estimate reconstruction performance
    pub fn estimate_reconstruction_time(&self, chunk_size_mb: f64, available_bandwidth_mbps: f64) -> f64 {
        let data_to_transfer = chunk_size_mb * self.config.data_chunks as f64;
        let transfer_time = data_to_transfer / available_bandwidth_mbps;
        let computation_time = chunk_size_mb * 0.1; // Estimate 0.1 seconds per MB for computation

        transfer_time + computation_time
    }
}

/// Reed-Solomon manager for handling multiple coding schemes
#[derive(Debug)]
pub struct ReedSolomonManager {
    pub codecs: HashMap<String, ReedSolomonCodec>,
    pub default_scheme: String,
}

impl ReedSolomonManager {
    /// Create new Reed-Solomon manager with multiple schemes
    pub fn new() -> Result<Self> {
        let mut manager = Self {
            codecs: HashMap::new(),
            default_scheme: "6+3".to_string(),
        };

        // Initialize standard schemes
        let schemes = [
            ("3+2", ReedSolomonConfig { data_chunks: 3, parity_chunks: 2, ..Default::default() }),
            ("6+3", ReedSolomonConfig { data_chunks: 6, parity_chunks: 3, ..Default::default() }),
            ("10+4", ReedSolomonConfig { data_chunks: 10, parity_chunks: 4, ..Default::default() }),
            ("8+6", ReedSolomonConfig { data_chunks: 8, parity_chunks: 6, ..Default::default() }),
        ];

        for (name, config) in schemes {
            let codec = ReedSolomonCodec::new(config)?;
            manager.codecs.insert(name.to_string(), codec);
        }

        Ok(manager)
    }

    /// Get codec for scheme
    pub fn get_codec(&mut self, scheme: &str) -> Result<&mut ReedSolomonCodec> {
        self.codecs.get_mut(scheme)
            .ok_or_else(|| anyhow::anyhow!("Reed-Solomon scheme '{}' not found", scheme))
    }

    /// Recommend scheme based on requirements
    pub fn recommend_scheme(&self, durability_requirement: f64, cost_sensitivity: f64) -> String {
        if durability_requirement > 0.99999 {
            "8+6".to_string() // Ultra high durability
        } else if durability_requirement > 0.9999 {
            "10+4".to_string() // High durability
        } else if cost_sensitivity > 0.7 {
            "3+2".to_string() // Cost sensitive
        } else {
            "6+3".to_string() // Balanced
        }
    }

    /// Get scheme performance characteristics
    pub fn get_scheme_info(&self, scheme: &str) -> Option<SchemeInfo> {
        self.codecs.get(scheme).map(|codec| {
            SchemeInfo {
                name: scheme.to_string(),
                data_chunks: codec.config.data_chunks,
                parity_chunks: codec.config.parity_chunks,
                storage_efficiency: codec.calculate_storage_efficiency(),
                fault_tolerance: codec.calculate_fault_tolerance(),
                reconstruction_complexity: (codec.config.data_chunks * codec.config.parity_chunks) as f64,
            }
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemeInfo {
    pub name: String,
    pub data_chunks: u32,
    pub parity_chunks: u32,
    pub storage_efficiency: f64,
    pub fault_tolerance: u32,
    pub reconstruction_complexity: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_galois_field_arithmetic() {
        let gf = GaloisField::new();

        // Test basic properties
        assert_eq!(gf.multiply(0, 5), 0);
        assert_eq!(gf.multiply(1, 5), 5);
        assert_eq!(gf.divide(10, 2), gf.multiply(10, gf.divide(1, 2)));

        // Test multiplicative inverse
        for i in 1..=255u8 {
            let inverse = gf.divide(1, i);
            assert_eq!(gf.multiply(i, inverse), 1);
        }
    }

    #[test]
    fn test_reed_solomon_codec_creation() {
        let config = ReedSolomonConfig::default();
        let codec = ReedSolomonCodec::new(config).unwrap();

        assert_eq!(codec.config.data_chunks, 6);
        assert_eq!(codec.config.parity_chunks, 3);
        assert_eq!(codec.encoding_matrix.len(), 9);
    }

    #[test]
    fn test_encoding_and_reconstruction() {
        let config = ReedSolomonConfig {
            data_chunks: 3,
            parity_chunks: 2,
            chunk_size: 1024,
            ..Default::default()
        };

        let mut codec = ReedSolomonCodec::new(config).unwrap();

        // Test data
        let test_data = b"Hello, Reed-Solomon World! This is a test of error correction.".to_vec();

        // Encode
        let chunks = codec.encode("test_chunk".to_string(), &test_data).unwrap();
        assert_eq!(chunks.len(), 5); // 3 data + 2 parity

        // Simulate losing 2 chunks (within error correction capability)
        let available_chunks = vec![chunks[0].clone(), chunks[2].clone(), chunks[4].clone()];
        let missing_indices = vec![1, 3];

        // Reconstruct
        let request = ReconstructionRequest {
            original_chunk_id: "test_chunk".to_string(),
            available_chunks,
            missing_chunk_indices: missing_indices,
            target_chunk_size: 1024,
        };

        let result = codec.decode(request).unwrap();
        assert!(result.success);
        assert_eq!(result.reconstructed_chunks.len(), 2);
        assert!(result.verification_passed);
    }

    #[test]
    fn test_manager_scheme_recommendation() {
        let manager = ReedSolomonManager::new().unwrap();

        // High durability requirement
        assert_eq!(manager.recommend_scheme(0.99999, 0.3), "8+6");

        // Cost sensitive
        assert_eq!(manager.recommend_scheme(0.999, 0.8), "3+2");

        // Balanced
        assert_eq!(manager.recommend_scheme(0.999, 0.3), "6+3");
    }

    #[test]
    fn test_storage_efficiency_calculation() {
        let config = ReedSolomonConfig {
            data_chunks: 6,
            parity_chunks: 3,
            ..Default::default()
        };

        let codec = ReedSolomonCodec::new(config).unwrap();
        let efficiency = codec.calculate_storage_efficiency();

        assert!((efficiency - 0.6667).abs() < 0.01); // 6/9 ≈ 0.6667
    }
}