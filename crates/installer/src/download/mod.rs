//! Download management with progress tracking.

// mod manager;
// mod verifier;
mod github;

// pub use manager::HttpDownloadManager;
// pub use verifier::FileVerifier;
pub use github::{GitHubAsset, GitHubRelease, GitHubReleases};
