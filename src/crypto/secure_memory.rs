//! Secure memory handling for ZephyrFS cryptographic operations
//!
//! Provides zeroizing containers for sensitive data like keys, passwords, and plaintext.
//! Ensures memory is securely cleared when data is no longer needed.

use zeroize::{Zeroize, ZeroizeOnDrop};
use std::ops::{Deref, DerefMut};
use std::fmt;

/// Secure byte buffer that automatically zeros memory on drop
#[derive(Clone, ZeroizeOnDrop)]
pub struct SecureBytes {
    data: Vec<u8>,
}

impl SecureBytes {
    /// Create new secure buffer with specified size
    pub fn new(size: usize) -> Self {
        Self {
            data: vec![0u8; size],
        }
    }

    /// Create from existing data (data will be zeroized in source)
    pub fn from_vec(mut data: Vec<u8>) -> Self {
        let secure = Self { data: data.clone() };
        data.zeroize(); // Clear original
        secure
    }

    /// Create from slice (slice data will be copied)
    pub fn from_slice(data: &[u8]) -> Self {
        Self {
            data: data.to_vec(),
        }
    }

    /// Get length of secure buffer
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Get immutable reference to bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Get mutable reference to bytes
    pub fn as_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }

    /// Copy data from slice into this buffer
    pub fn copy_from_slice(&mut self, src: &[u8]) {
        if src.len() != self.data.len() {
            panic!("Source length {} doesn't match buffer length {}", src.len(), self.data.len());
        }
        self.data.copy_from_slice(src);
    }

    /// Resize buffer (new space will be zero-filled)
    pub fn resize(&mut self, new_size: usize) {
        self.data.resize(new_size, 0);
    }

    /// Clear buffer contents (set all bytes to zero)
    pub fn clear(&mut self) {
        self.data.zeroize();
    }

    /// Split buffer at index, returning (left, right)
    pub fn split_at(mut self, mid: usize) -> (SecureBytes, SecureBytes) {
        let right = self.data.split_off(mid);
        let left_data = std::mem::take(&mut self.data);
        let left = SecureBytes { data: left_data };
        let right = SecureBytes { data: right };
        self.data.zeroize(); // Clear original (now empty)
        (left, right)
    }

    /// Extend buffer with data from slice
    pub fn extend_from_slice(&mut self, data: &[u8]) {
        self.data.extend_from_slice(data);
    }

    /// Convert to Vec<u8> (consumes self and zeros original)
    pub fn into_vec(mut self) -> Vec<u8> {
        let data = std::mem::take(&mut self.data);
        self.data.zeroize();
        data
    }
}

impl Deref for SecureBytes {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl DerefMut for SecureBytes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl AsRef<[u8]> for SecureBytes {
    fn as_ref(&self) -> &[u8] {
        &self.data
    }
}

impl AsMut<[u8]> for SecureBytes {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }
}

// Implement Debug but don't show actual data
impl fmt::Debug for SecureBytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SecureBytes")
            .field("len", &self.data.len())
            .field("data", &"[REDACTED]")
            .finish()
    }
}

// Implement PartialEq for testing, but use constant-time comparison
impl PartialEq for SecureBytes {
    fn eq(&self, other: &Self) -> bool {
        if self.data.len() != other.data.len() {
            return false;
        }
        constant_time_eq::constant_time_eq(&self.data, &other.data)
    }
}

impl Eq for SecureBytes {}

impl Zeroize for SecureBytes {
    fn zeroize(&mut self) {
        self.data.zeroize();
    }
}

/// Secure string that automatically zeros memory on drop
#[derive(Clone, ZeroizeOnDrop)]
pub struct SecureString {
    data: String,
}

impl SecureString {
    /// Create new empty secure string
    pub fn new() -> Self {
        Self {
            data: String::new(),
        }
    }

    /// Create from existing string (string will be zeroized in source)
    pub fn from_string(mut data: String) -> Self {
        let secure = Self { data: data.clone() };
        data.zeroize();
        secure
    }

    /// Create from str (data will be copied)
    pub fn from_str(data: &str) -> Self {
        Self {
            data: data.to_string(),
        }
    }

    /// Get length of string
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if string is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Get str reference
    pub fn as_str(&self) -> &str {
        &self.data
    }

    /// Push character to string
    pub fn push(&mut self, ch: char) {
        self.data.push(ch);
    }

    /// Push str to string
    pub fn push_str(&mut self, string: &str) {
        self.data.push_str(string);
    }

    /// Clear string contents
    pub fn clear(&mut self) {
        self.data.zeroize();
        self.data.clear();
    }

    /// Convert to String (consumes self and zeros original)
    pub fn into_string(mut self) -> String {
        let data = std::mem::take(&mut self.data);
        self.data.zeroize();
        data
    }

    /// Get bytes of the UTF-8 string
    pub fn as_bytes(&self) -> &[u8] {
        self.data.as_bytes()
    }
}

impl Default for SecureString {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for SecureString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl AsRef<str> for SecureString {
    fn as_ref(&self) -> &str {
        &self.data
    }
}

impl fmt::Debug for SecureString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SecureString")
            .field("len", &self.data.len())
            .field("data", &"[REDACTED]")
            .finish()
    }
}

impl PartialEq for SecureString {
    fn eq(&self, other: &Self) -> bool {
        constant_time_eq::constant_time_eq(self.data.as_bytes(), other.data.as_bytes())
    }
}

impl Eq for SecureString {}

impl Zeroize for SecureString {
    fn zeroize(&mut self) {
        self.data.zeroize();
    }
}

/// Utility for securely reading passwords from console
pub struct SecureInput;

impl SecureInput {
    /// Read password from stdin without echoing (requires rpassword crate)
    #[cfg(feature = "password-input")]
    pub fn read_password(prompt: &str) -> std::io::Result<SecureString> {
        use rpassword::read_password_from_tty;
        print!("{}", prompt);
        std::io::flush(std::io::stdout())?;
        let password = read_password_from_tty(None)?;
        Ok(SecureString::from_string(password))
    }

    /// Read password from stdin (fallback - echoes input, for testing only)
    #[cfg(not(feature = "password-input"))]
    pub fn read_password(prompt: &str) -> std::io::Result<SecureString> {
        use std::io::{self, Write};
        print!("{}", prompt);
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        // Remove trailing newline
        if input.ends_with('\n') {
            input.pop();
        }
        
        Ok(SecureString::from_string(input))
    }
}

/// Helper trait for creating secure versions of sensitive data
pub trait IntoSecure {
    type Output;
    fn into_secure(self) -> Self::Output;
}

impl IntoSecure for Vec<u8> {
    type Output = SecureBytes;
    
    fn into_secure(self) -> Self::Output {
        SecureBytes::from_vec(self)
    }
}

impl IntoSecure for String {
    type Output = SecureString;
    
    fn into_secure(self) -> Self::Output {
        SecureString::from_string(self)
    }
}

impl IntoSecure for &[u8] {
    type Output = SecureBytes;
    
    fn into_secure(self) -> Self::Output {
        SecureBytes::from_slice(self)
    }
}

impl IntoSecure for &str {
    type Output = SecureString;
    
    fn into_secure(self) -> Self::Output {
        SecureString::from_str(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secure_bytes_basic_operations() {
        let mut secure = SecureBytes::new(32);
        assert_eq!(secure.len(), 32);
        assert!(!secure.is_empty());

        // Fill with test data
        for (i, byte) in secure.as_mut().iter_mut().enumerate() {
            *byte = (i % 256) as u8;
        }

        // Check data
        for (i, &byte) in secure.as_bytes().iter().enumerate() {
            assert_eq!(byte, (i % 256) as u8);
        }

        // Clear should zero all data
        secure.clear();
        for &byte in secure.as_bytes() {
            assert_eq!(byte, 0);
        }
    }

    #[test]
    fn test_secure_bytes_from_slice() {
        let data = b"Hello, ZephyrFS!";
        let secure = SecureBytes::from_slice(data);
        
        assert_eq!(secure.len(), data.len());
        assert_eq!(secure.as_bytes(), data);
    }

    #[test]
    fn test_secure_bytes_split() {
        let mut secure = SecureBytes::new(10);
        for (i, byte) in secure.as_mut().iter_mut().enumerate() {
            *byte = i as u8;
        }

        let (left, right) = secure.split_at(4);
        assert_eq!(left.len(), 4);
        assert_eq!(right.len(), 6);
        assert_eq!(left.as_bytes(), &[0, 1, 2, 3]);
        assert_eq!(right.as_bytes(), &[4, 5, 6, 7, 8, 9]);
    }

    #[test]
    fn test_secure_string_operations() {
        let mut secure = SecureString::from_str("Hello");
        assert_eq!(secure.len(), 5);
        assert_eq!(secure.as_str(), "Hello");

        secure.push_str(", World!");
        assert_eq!(secure.as_str(), "Hello, World!");

        secure.clear();
        assert!(secure.is_empty());
    }

    #[test]
    fn test_secure_memory_equality() {
        let secure1 = SecureBytes::from_slice(b"test data");
        let secure2 = SecureBytes::from_slice(b"test data");
        let secure3 = SecureBytes::from_slice(b"different");

        assert_eq!(secure1, secure2);
        assert_ne!(secure1, secure3);

        let str1 = SecureString::from_str("password");
        let str2 = SecureString::from_str("password");
        let str3 = SecureString::from_str("different");

        assert_eq!(str1, str2);
        assert_ne!(str1, str3);
    }

    #[test] 
    fn test_into_secure_trait() {
        let vec = vec![1, 2, 3, 4, 5];
        let secure_bytes = vec.into_secure();
        assert_eq!(secure_bytes.as_bytes(), &[1, 2, 3, 4, 5]);

        let string = "test string".to_string();
        let secure_string = string.into_secure();
        assert_eq!(secure_string.as_str(), "test string");

        let slice: &[u8] = &[10, 20, 30];
        let secure_bytes = slice.into_secure();
        assert_eq!(secure_bytes.as_bytes(), &[10, 20, 30]);

        let str_ref: &str = "reference string";
        let secure_string = str_ref.into_secure();
        assert_eq!(secure_string.as_str(), "reference string");
    }

    #[test]
    fn test_debug_redaction() {
        let secure_bytes = SecureBytes::from_slice(b"secret data");
        let debug_str = format!("{:?}", secure_bytes);
        assert!(debug_str.contains("[REDACTED]"));
        assert!(!debug_str.contains("secret"));

        let secure_string = SecureString::from_str("secret password");
        let debug_str = format!("{:?}", secure_string);
        assert!(debug_str.contains("[REDACTED]"));
        assert!(!debug_str.contains("password"));
    }

    #[test]
    fn test_zeroize_on_drop() {
        let mut data = vec![1, 2, 3, 4, 5];
        {
            let _secure = SecureBytes::from_vec(data.clone());
            // secure goes out of scope here and should zero its memory
        }
        // Original data should be zeroed
        assert_eq!(data, vec![0, 0, 0, 0, 0]);
    }
}