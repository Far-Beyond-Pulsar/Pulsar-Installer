//! File extraction step.

use crate::traits::{InstallStep, ProgressCallback};
use crate::Result;
use async_trait::async_trait;
use flate2::read::GzDecoder;
use std::fs::File;
use std::path::{Path, PathBuf};
use tar::Archive;

/// Installation step that extracts archive files.
pub struct ExtractFilesStep {
    archive_path: PathBuf,
    destination: PathBuf,
}

impl ExtractFilesStep {
    /// Create a new file extraction step.
    pub fn new(archive_path: PathBuf, destination: PathBuf) -> Self {
        Self {
            archive_path,
            destination,
        }
    }

    /// Extract a tar.gz archive.
    fn extract_tar_gz(&self, progress: &ProgressCallback) -> Result<()> {
        let file = File::open(&self.archive_path)?;
        let tar = GzDecoder::new(file);
        let mut archive = Archive::new(tar);

        // Get total entries for progress tracking
        let entries: Vec<_> = archive.entries()?.collect();
        let total = entries.len() as f32;

        // Re-open the archive for extraction
        let file = File::open(&self.archive_path)?;
        let tar = GzDecoder::new(file);
        let mut archive = Archive::new(tar);

        for (i, entry) in archive.entries()?.enumerate() {
            let mut entry = entry?;
            entry.unpack_in(&self.destination)?;

            let percent = ((i + 1) as f32 / total) * 100.0;
            progress(crate::traits::Progress::new(percent));
        }

        Ok(())
    }
}

#[async_trait]
impl InstallStep for ExtractFilesStep {
    fn name(&self) -> &str {
        "Extract Files"
    }

    fn description(&self) -> &str {
        "Extracting Pulsar game engine files"
    }

    async fn execute(&self, progress: ProgressCallback) -> Result<()> {
        progress(crate::traits::Progress::new(0.0).with_message("Extracting files..."));

        // Determine archive type and extract
        let extension = self
            .archive_path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        match extension {
            "gz" | "tgz" => self.extract_tar_gz(&progress)?,
            _ => {
                return Err(crate::error::InstallerError::Other(format!(
                    "Unsupported archive format: {}",
                    extension
                )))
            }
        }

        progress(crate::traits::Progress::new(100.0).with_message("Files extracted"));

        Ok(())
    }

    async fn rollback(&self) -> Result<()> {
        // This would be handled by the CreateDirectoriesStep rollback
        Ok(())
    }
}
