//! Directory creation step.

use crate::traits::{InstallStep, ProgressCallback};
use crate::Result;
use async_trait::async_trait;
use std::path::PathBuf;

/// Installation step that creates necessary directories.
pub struct CreateDirectoriesStep {
    base_path: PathBuf,
    subdirectories: Vec<String>,
}

impl CreateDirectoriesStep {
    /// Create a new directory creation step.
    pub fn new(base_path: PathBuf) -> Self {
        Self {
            base_path,
            subdirectories: vec![
                "bin".to_string(),
                "lib".to_string(),
                "assets".to_string(),
                "plugins".to_string(),
                "projects".to_string(),
                "docs".to_string(),
            ],
        }
    }

    /// Add additional subdirectories to create.
    pub fn with_subdirectories(mut self, dirs: Vec<String>) -> Self {
        self.subdirectories.extend(dirs);
        self
    }
}

#[async_trait]
impl InstallStep for CreateDirectoriesStep {
    fn name(&self) -> &str {
        "Create Directories"
    }

    fn description(&self) -> &str {
        "Creating installation directory structure"
    }

    async fn execute(&self, progress: ProgressCallback) -> Result<()> {
        // Create base directory
        progress(crate::traits::Progress::new(0.0).with_message("Creating base directory..."));
        std::fs::create_dir_all(&self.base_path)?;

        let total = self.subdirectories.len() as f32;
        for (i, subdir) in self.subdirectories.iter().enumerate() {
            let dir_path = self.base_path.join(subdir);
            std::fs::create_dir_all(&dir_path)?;

            let percent = ((i + 1) as f32 / total) * 100.0;
            progress(crate::traits::Progress::new(percent));
        }

        progress(crate::traits::Progress::new(100.0).with_message("Directories created"));

        Ok(())
    }

    async fn rollback(&self) -> Result<()> {
        // Remove the entire installation directory
        if self.base_path.exists() {
            std::fs::remove_dir_all(&self.base_path)?;
        }
        Ok(())
    }
}
