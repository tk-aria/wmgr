/// Infrastructure layer modules
/// 
/// This layer provides concrete implementations for external system interactions:
/// - Git operations (clone, fetch, push, etc.)
/// - File system operations (config files, manifests)
/// - Process execution (command runners, parallel execution)

pub mod git;
pub mod filesystem;
pub mod process;

// Re-export commonly used types
pub use git::{GitRepository, GitRemoteManager};
pub use filesystem::{config_store::ConfigStore, manifest_store::ManifestStore};
pub use process::CommandExecutor;