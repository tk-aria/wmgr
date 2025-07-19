pub mod filesystem;
/// Infrastructure layer modules
///
/// This layer provides concrete implementations for external system interactions:
/// - Git operations (clone, fetch, push, etc.)
/// - File system operations (config files, manifests)
/// - Process execution (command runners, parallel execution)
pub mod git;
pub mod process;

// Re-export commonly used types
pub use filesystem::{config_store::ConfigStore, manifest_store::ManifestStore};
pub use git::{GitRemoteManager, GitRepository};
pub use process::CommandExecutor;
