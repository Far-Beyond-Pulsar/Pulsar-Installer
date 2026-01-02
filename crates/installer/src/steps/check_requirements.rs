//! System requirements checking step.

use crate::traits::{InstallStep, ProgressCallback, SystemDetector, SystemRequirements};
use crate::Result;
use async_trait::async_trait;
use std::sync::Arc;

/// Installation step that verifies system requirements.
pub struct CheckRequirementsStep {
    detector: Arc<dyn SystemDetector>,
    requirements: SystemRequirements,
}

impl CheckRequirementsStep {
    /// Create a new requirements checking step.
    pub fn new(detector: Arc<dyn SystemDetector>, requirements: SystemRequirements) -> Self {
        Self {
            detector,
            requirements,
        }
    }
}

#[async_trait]
impl InstallStep for CheckRequirementsStep {
    fn name(&self) -> &str {
        "Check System Requirements"
    }

    fn description(&self) -> &str {
        "Verifying that your system meets the minimum requirements for Pulsar"
    }

    async fn execute(&self, progress: ProgressCallback) -> Result<()> {
        progress(crate::traits::Progress::new(0.0).with_message("Checking system information..."));

        // Check requirements
        self.detector.check_requirements(&self.requirements).await?;

        progress(crate::traits::Progress::new(100.0).with_message("System requirements verified"));

        Ok(())
    }
}
