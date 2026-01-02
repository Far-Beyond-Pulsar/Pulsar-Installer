//! Pre-built installation steps for common operations.

mod check_requirements;
mod create_directories;
mod extract_files;
mod create_shortcuts;
mod finalize;

pub use check_requirements::CheckRequirementsStep;
pub use create_directories::CreateDirectoriesStep;
pub use extract_files::ExtractFilesStep;
pub use create_shortcuts::CreateShortcutsStep;
pub use finalize::FinalizeStep;

use crate::traits::InstallStep;
use crate::Result;
use std::sync::Arc;

/// A collection of installation steps that execute sequentially.
pub struct StepSequence {
    steps: Vec<Arc<dyn InstallStep>>,
    current_step: usize,
}

impl StepSequence {
    /// Create a new step sequence.
    pub fn new() -> Self {
        Self {
            steps: Vec::new(),
            current_step: 0,
        }
    }

    /// Add a step to the sequence.
    pub fn add_step(mut self, step: Arc<dyn InstallStep>) -> Self {
        self.steps.push(step);
        self
    }

    /// Get all steps in this sequence.
    pub fn steps(&self) -> &[Arc<dyn InstallStep>] {
        &self.steps
    }

    /// Get the current step index.
    pub fn current_step(&self) -> usize {
        self.current_step
    }

    /// Get the total number of steps.
    pub fn total_steps(&self) -> usize {
        self.steps.len()
    }

    /// Check if all steps are complete.
    pub fn is_complete(&self) -> bool {
        self.current_step >= self.steps.len()
    }
}

impl Default for StepSequence {
    fn default() -> Self {
        Self::new()
    }
}
