//! File verification using checksums.

use crate::error::{InstallerError, Result};
use sha2::{Digest, Sha256};
use std::path::Path;

/// File verification utility.
pub struct FileVerifier;

impl FileVerifier {
    /// Create a new file verifier.
    pub fn new() -> Self {
        Self
    }

    /// Calculate SHA256 checksum of a file.
    pub async fn calculate_sha256(&self, path: &Path) -> Result<String> {
        let data = smol::fs::read(path).await?;
        let hash = Sha256::digest(&data);
        Ok(hex::encode(hash))
    }

    /// Verify file checksum against expected value.
    pub async fn verify_sha256(&self, path: &Path, expected: &str) -> Result<()> {
        let actual = self.calculate_sha256(path).await?;

        if actual.to_lowercase() != expected.to_lowercase() {
            return Err(InstallerError::ChecksumMismatch {
                file: path.display().to_string(),
                expected: expected.to_string(),
                actual,
            });
        }

        Ok(())
    }
}

impl Default for FileVerifier {
    fn default() -> Self {
        Self::new()
    }
}
