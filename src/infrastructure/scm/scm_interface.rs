use crate::domain::value_objects::scm_type::ScmType;
use async_trait::async_trait;
use std::any::Any;
use std::path::Path;

/// Helper trait to enable downcasting
pub trait AsAny {
    fn as_any(&self) -> &dyn Any;
}

/// Common interface for all SCM operations
#[async_trait]
pub trait ScmOperations: AsAny + Send + Sync {
    /// Clone a repository from the given URL to the specified path
    async fn clone_repository(
        &self,
        url: &str,
        dest_path: &Path,
        options: &CloneOptions,
    ) -> Result<(), ScmError>;

    /// Synchronize (update) an existing repository
    async fn sync_repository(
        &self,
        repo_path: &Path,
        options: &SyncOptions,
    ) -> Result<(), ScmError>;

    /// Get the status of a repository
    async fn get_status(&self, repo_path: &Path) -> Result<StatusResult, ScmError>;

    /// Check if a directory is a valid repository for this SCM
    fn is_repository(&self, path: &Path) -> bool;

    /// Get the SCM type this implementation handles
    fn scm_type(&self) -> ScmType;

    /// Get the current revision/commit identifier
    async fn get_current_revision(&self, repo_path: &Path) -> Result<String, ScmError>;

    /// Check if the repository has uncommitted changes
    async fn has_changes(&self, repo_path: &Path) -> Result<bool, ScmError>;
}

/// Options for cloning repositories
#[derive(Debug, Clone)]
pub struct CloneOptions {
    /// Branch to clone (Git only)
    pub branch: Option<String>,
    /// Perform shallow clone (Git only)
    pub shallow: bool,
    /// Shallow clone depth (Git only)
    pub depth: Option<u32>,
    /// Remote name (Git only)
    pub remote: Option<String>,
    /// Recurse submodules (Git only)
    pub recurse_submodules: bool,
    /// Specific revision to checkout
    pub revision: Option<String>,
    /// Username for authentication
    pub username: Option<String>,
    /// Password for authentication
    pub password: Option<String>,
    /// Client workspace (P4 only)
    pub client: Option<String>,
    /// Stream (P4 only)
    pub stream: Option<String>,
    /// Additional SCM-specific options
    pub extra_options: Vec<String>,
}

impl Default for CloneOptions {
    fn default() -> Self {
        Self {
            branch: None,
            shallow: false,
            depth: None,
            remote: None,
            recurse_submodules: false,
            revision: None,
            username: None,
            password: None,
            client: None,
            stream: None,
            extra_options: Vec::new(),
        }
    }
}

/// Options for synchronizing repositories
#[derive(Debug, Clone)]
pub struct SyncOptions {
    /// Target branch (Git only)
    pub branch: Option<String>,
    /// Force update, discarding local changes
    pub force: bool,
    /// Target revision to update to
    pub revision: Option<String>,
    /// Username for authentication
    pub username: Option<String>,
    /// Password for authentication
    pub password: Option<String>,
    /// Client workspace (P4 only)
    pub client: Option<String>,
    /// Stream (P4 only)
    pub stream: Option<String>,
    /// Additional SCM-specific options
    pub extra_options: Vec<String>,
}

impl Default for SyncOptions {
    fn default() -> Self {
        Self {
            branch: None,
            force: false,
            revision: None,
            username: None,
            password: None,
            client: None,
            stream: None,
            extra_options: Vec::new(),
        }
    }
}

/// Result of status check operation
#[derive(Debug, Clone)]
pub struct StatusResult {
    /// Current revision/commit identifier
    pub current_revision: String,
    /// Current branch (Git only)
    pub current_branch: Option<String>,
    /// Whether there are uncommitted changes
    pub has_changes: bool,
    /// Whether there are untracked files
    pub has_untracked: bool,
    /// Number of files ahead of remote (Git only)
    pub ahead_count: Option<usize>,
    /// Number of files behind remote (Git only)
    pub behind_count: Option<usize>,
    /// SCM-specific status information
    pub extra_info: std::collections::HashMap<String, String>,
}

/// Errors that can occur during SCM operations
#[derive(Debug, thiserror::Error)]
pub enum ScmError {
    #[error("Repository not found at path: {path}")]
    RepositoryNotFound { path: String },

    #[error("Invalid repository format for {scm_type} at path: {path}")]
    InvalidRepository { scm_type: ScmType, path: String },

    #[error("Clone operation failed: {message}")]
    CloneFailed { message: String },

    #[error("Sync operation failed: {message}")]
    SyncFailed { message: String },

    #[error("Status check failed: {message}")]
    StatusFailed { message: String },

    #[error("Authentication failed for {url}")]
    AuthenticationFailed { url: String },

    #[error("Network error: {message}")]
    NetworkError { message: String },

    #[error("SCM executable not found: {executable}")]
    ExecutableNotFound { executable: String },

    #[error("Unsupported operation for {scm_type}: {operation}")]
    UnsupportedOperation { scm_type: ScmType, operation: String },

    #[error("Permission denied: {message}")]
    PermissionDenied { message: String },

    #[error("Invalid URL format: {url}")]
    InvalidUrl { url: String },

    #[error("IO error: {source}")]
    IoError {
        #[from]
        source: std::io::Error,
    },

    #[error("Command execution failed: {command}, exit code: {exit_code}, stderr: {stderr}")]
    CommandFailed {
        command: String,
        exit_code: i32,
        stderr: String,
    },

    #[error("Internal error: {message}")]
    Internal { message: String },
}

impl ScmError {
    /// Create a clone failed error
    pub fn clone_failed(message: impl Into<String>) -> Self {
        Self::CloneFailed {
            message: message.into(),
        }
    }

    /// Create a sync failed error
    pub fn sync_failed(message: impl Into<String>) -> Self {
        Self::SyncFailed {
            message: message.into(),
        }
    }

    /// Create a status failed error
    pub fn status_failed(message: impl Into<String>) -> Self {
        Self::StatusFailed {
            message: message.into(),
        }
    }

    /// Create a network error
    pub fn network_error(message: impl Into<String>) -> Self {
        Self::NetworkError {
            message: message.into(),
        }
    }

    /// Create an authentication failed error
    pub fn auth_failed(url: impl Into<String>) -> Self {
        Self::AuthenticationFailed { url: url.into() }
    }

    /// Create an executable not found error
    pub fn executable_not_found(executable: impl Into<String>) -> Self {
        Self::ExecutableNotFound {
            executable: executable.into(),
        }
    }

    /// Create an unsupported operation error
    pub fn unsupported_operation(scm_type: ScmType, operation: impl Into<String>) -> Self {
        Self::UnsupportedOperation {
            scm_type,
            operation: operation.into(),
        }
    }

    /// Create a command failed error
    pub fn command_failed(
        command: impl Into<String>,
        exit_code: i32,
        stderr: impl Into<String>,
    ) -> Self {
        Self::CommandFailed {
            command: command.into(),
            exit_code,
            stderr: stderr.into(),
        }
    }
}