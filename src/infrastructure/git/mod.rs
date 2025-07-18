pub mod repository;
pub mod remote;

// Re-export main types for convenience
pub use repository::{
    GitRepository, GitRepositoryError, RepositoryStatus, CloneConfig, FetchConfig,
    ResetMode, GitBranchType,
};
pub use remote::{
    GitRemoteManager, GitRemoteError, RemoteInfo,
};