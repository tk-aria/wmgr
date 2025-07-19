pub mod remote;
pub mod repository;

// Re-export main types for convenience
pub use remote::{GitRemoteError, GitRemoteManager, RemoteInfo};
pub use repository::{
    CloneConfig, FetchConfig, GitBranchType, GitRepository, GitRepositoryError, RepositoryStatus,
    ResetMode,
};
