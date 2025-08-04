pub mod filesystem;
/// Infrastructure layer modules
///
/// This layer provides concrete implementations for external system interactions:
/// - SCM operations (Git, SVN, Perforce)
/// - Git operations (clone, fetch, push, etc.)
/// - File system operations (config files, manifests)
/// - Process execution (command runners, parallel execution)
pub mod git;
pub mod process;
pub mod scm;

// Re-export commonly used types
pub use filesystem::{config_store::ConfigStore, manifest_store::ManifestStore};
pub use git::{GitRemoteManager, GitRepository};
pub use process::CommandExecutor;
pub use scm::{
    scm_factory::ScmFactory,
    scm_interface::{AsAny, CloneOptions, ScmError, ScmOperations, StatusResult, SyncOptions},
};
