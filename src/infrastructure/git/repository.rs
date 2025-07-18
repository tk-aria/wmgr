use std::path::{Path, PathBuf};
use thiserror::Error;
use git2::{
    Repository as Git2Repository, FetchOptions, RemoteCallbacks,
    BranchType, Oid, ResetType, Cred, CredentialType,
    build::CheckoutBuilder,
};
use crate::domain::value_objects::{
    git_url::{GitUrl, GitUrlError},
    branch_name::{BranchName, BranchNameError},
    file_path::{FilePath, FilePathError},
};

/// Git repository operations related errors
#[derive(Debug, Error)]
pub enum GitRepositoryError {
    #[error("Repository not found at path: {0}")]
    RepositoryNotFound(String),
    
    #[error("Invalid repository path: {0}")]
    InvalidRepositoryPath(String),
    
    #[error("Git clone failed: {0}")]
    CloneFailed(String),
    
    #[error("Git fetch failed: {0}")]
    FetchFailed(String),
    
    #[error("Git checkout failed: {0}")]
    CheckoutFailed(String),
    
    #[error("Git reset failed: {0}")]
    ResetFailed(String),
    
    #[error("Git merge failed: {0}")]
    MergeFailed(String),
    
    #[error("Branch not found: {0}")]
    BranchNotFound(String),
    
    #[error("Remote not found: {0}")]
    RemoteNotFound(String),
    
    #[error("Git operation failed: {0}")]
    GitOperationFailed(String),
    
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    
    #[error("Working directory has uncommitted changes")]
    WorkingDirectoryDirty,
    
    #[error("Git URL error: {0}")]
    GitUrlError(#[from] GitUrlError),
    
    #[error("Branch name error: {0}")]
    BranchNameError(#[from] BranchNameError),
    
    #[error("File path error: {0}")]
    FilePathError(#[from] FilePathError),
    
    #[error("Git2 error: {0}")]
    Git2Error(#[from] git2::Error),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Git repository status information
#[derive(Debug, Clone)]
pub struct RepositoryStatus {
    /// Current branch name
    pub current_branch: Option<String>,
    
    /// Current commit SHA
    pub current_commit: String,
    
    /// Whether the working directory is clean
    pub is_clean: bool,
    
    /// Number of ahead commits from upstream
    pub ahead: usize,
    
    /// Number of behind commits from upstream
    pub behind: usize,
    
    /// List of modified files
    pub modified_files: Vec<String>,
    
    /// List of untracked files
    pub untracked_files: Vec<String>,
    
    /// List of staged files
    pub staged_files: Vec<String>,
}

/// Clone options for repository cloning
#[derive(Debug, Clone)]
pub struct CloneConfig {
    /// Target branch to clone
    pub branch: Option<String>,
    
    /// Whether to perform a shallow clone
    pub shallow: bool,
    
    /// Depth for shallow clone (if shallow is true)
    pub depth: Option<i32>,
    
    /// Whether to clone recursively (submodules)
    pub recursive: bool,
    
    /// Progress callback during clone
    pub progress_callback: Option<fn(&str, usize, usize)>,
}

impl Default for CloneConfig {
    fn default() -> Self {
        Self {
            branch: None,
            shallow: false,
            depth: None,
            recursive: false,
            progress_callback: None,
        }
    }
}

/// Fetch options for repository fetching
#[derive(Debug, Clone)]
pub struct FetchConfig {
    /// Remote name to fetch from
    pub remote_name: String,
    
    /// Specific refs to fetch (if None, fetch all)
    pub refs: Option<Vec<String>>,
    
    /// Progress callback during fetch
    pub progress_callback: Option<fn(&str, usize, usize)>,
}

impl Default for FetchConfig {
    fn default() -> Self {
        Self {
            remote_name: "origin".to_string(),
            refs: None,
            progress_callback: None,
        }
    }
}

/// Wrapper around git2::Repository with high-level operations
pub struct GitRepository {
    /// The underlying git2 repository
    repo: Git2Repository,
    
    /// Repository path
    path: PathBuf,
}

impl std::fmt::Debug for GitRepository {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GitRepository")
            .field("path", &self.path)
            .field("repo", &"<git2::Repository>")
            .finish()
    }
}

impl GitRepository {
    /// Open an existing Git repository
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, GitRepositoryError> {
        let path_buf = path.as_ref().to_path_buf();
        
        if !path_buf.exists() {
            return Err(GitRepositoryError::RepositoryNotFound(
                path_buf.display().to_string()
            ));
        }
        
        let repo = Git2Repository::open(&path_buf)
            .map_err(|e| GitRepositoryError::GitOperationFailed(e.to_string()))?;
            
        Ok(Self {
            repo,
            path: path_buf,
        })
    }
    
    /// Initialize a new Git repository
    pub fn init<P: AsRef<Path>>(path: P, bare: bool) -> Result<Self, GitRepositoryError> {
        let path_buf = path.as_ref().to_path_buf();
        
        // Create directory if it doesn't exist
        if !path_buf.exists() {
            std::fs::create_dir_all(&path_buf)?;
        }
        
        let repo = if bare {
            Git2Repository::init_bare(&path_buf)?
        } else {
            Git2Repository::init(&path_buf)?
        };
        
        Ok(Self {
            repo,
            path: path_buf,
        })
    }
    
    /// Clone a remote repository
    pub async fn clone(
        url: &GitUrl,
        target_path: &FilePath,
        config: CloneConfig,
    ) -> Result<Self, GitRepositoryError> {
        let target_path_buf = target_path.as_path().to_path_buf();
        
        // Create parent directories if they don't exist
        if let Some(parent) = target_path_buf.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        // Use git2's RepoBuilder for more control
        let mut builder = git2::build::RepoBuilder::new();
        
        // Set up fetch options
        let mut fetch_options = FetchOptions::new();
        let mut callbacks = RemoteCallbacks::new();
        
        // Set up authentication
        callbacks.credentials(|_url, username_from_url, allowed_types| {
            if allowed_types.contains(CredentialType::SSH_KEY) {
                // Try SSH key authentication
                Cred::ssh_key_from_agent(username_from_url.unwrap_or("git"))
            } else if allowed_types.contains(CredentialType::USER_PASS_PLAINTEXT) {
                // TODO: Handle username/password authentication
                Err(git2::Error::from_str("Username/password authentication not implemented"))
            } else {
                Err(git2::Error::from_str("No supported authentication method"))
            }
        });
        
        // Note: For progress callback, we'd need to store it somewhere accessible
        // For now, skipping the progress callback to avoid lifetime issues
        // TODO: Implement progress callback with proper lifetime management
        
        fetch_options.remote_callbacks(callbacks);
        builder.fetch_options(fetch_options);
        
        // Set branch if specified
        if let Some(branch) = &config.branch {
            builder.branch(branch);
        }
        
        // Convert GitUrl to appropriate format for git2
        let clone_url = url.to_https_url(); // Start with HTTPS, fallback to SSH if needed
        
        // Perform the clone
        let repo = builder.clone(&clone_url, &target_path_buf)?;
        
        Ok(Self {
            repo,
            path: target_path_buf,
        })
    }
    
    /// Fetch changes from remote
    pub async fn fetch(&self, config: FetchConfig) -> Result<(), GitRepositoryError> {
        let mut remote = self.repo.find_remote(&config.remote_name)
            .map_err(|_| GitRepositoryError::RemoteNotFound(config.remote_name.clone()))?;
        
        let mut fetch_options = FetchOptions::new();
        let mut callbacks = RemoteCallbacks::new();
        
        // Set up authentication
        callbacks.credentials(|_url, username_from_url, allowed_types| {
            if allowed_types.contains(CredentialType::SSH_KEY) {
                Cred::ssh_key_from_agent(username_from_url.unwrap_or("git"))
            } else {
                Err(git2::Error::from_str("Authentication not supported"))
            }
        });
        
        // Note: Progress callback is skipped for now due to lifetime complexity
        // TODO: Implement progress callback with proper lifetime management
        
        fetch_options.remote_callbacks(callbacks);
        
        // Determine refs to fetch
        let refs: Vec<&str> = if let Some(ref_list) = &config.refs {
            ref_list.iter().map(|s| s.as_str()).collect()
        } else {
            vec![] // Empty means fetch all
        };
        
        // Perform fetch
        remote.fetch(&refs, Some(&mut fetch_options), None)?;
        
        Ok(())
    }
    
    /// Checkout a specific branch or commit
    pub fn checkout(&self, target: &str) -> Result<(), GitRepositoryError> {
        // Try to find the reference
        let reference = self.repo.find_reference(target)
            .or_else(|_| self.repo.find_reference(&format!("refs/heads/{}", target)))
            .or_else(|_| self.repo.find_reference(&format!("refs/remotes/origin/{}", target)))
            .map_err(|_| GitRepositoryError::BranchNotFound(target.to_string()))?;
        
        let commit = reference.peel_to_commit()?;
        
        // Checkout the commit
        let mut checkout_builder = CheckoutBuilder::new();
        checkout_builder.safe();
        
        self.repo.checkout_tree(commit.as_object(), Some(&mut checkout_builder))?;
        
        // Update HEAD
        if reference.is_branch() {
            self.repo.set_head(reference.name().unwrap())?;
        } else {
            self.repo.set_head_detached(commit.id())?;
        }
        
        Ok(())
    }
    
    /// Create and checkout a new branch
    pub fn create_branch(&self, branch_name: &BranchName, start_point: Option<&str>) -> Result<(), GitRepositoryError> {
        let start_commit = if let Some(start) = start_point {
            // Find the start point commit
            let reference = self.repo.find_reference(start)
                .or_else(|_| self.repo.find_reference(&format!("refs/heads/{}", start)))
                .or_else(|_| self.repo.find_reference(&format!("refs/remotes/origin/{}", start)))
                .map_err(|_| GitRepositoryError::BranchNotFound(start.to_string()))?;
            reference.peel_to_commit()?
        } else {
            // Use HEAD
            self.repo.head()?.peel_to_commit()?
        };
        
        // Create the branch
        let _branch = self.repo.branch(branch_name.as_str(), &start_commit, false)?;
        
        // Checkout the new branch
        self.checkout(branch_name.as_str())?;
        
        Ok(())
    }
    
    /// Reset repository to a specific state
    pub fn reset(&self, target: &str, reset_type: ResetMode) -> Result<(), GitRepositoryError> {
        // Find the target commit
        let target_commit = if target.len() == 40 {
            // Assume it's a SHA
            let oid = Oid::from_str(target)
                .map_err(|_| GitRepositoryError::GitOperationFailed("Invalid SHA".to_string()))?;
            self.repo.find_commit(oid)?
        } else {
            // Assume it's a reference
            let reference = self.repo.find_reference(target)
                .or_else(|_| self.repo.find_reference(&format!("refs/heads/{}", target)))
                .or_else(|_| self.repo.find_reference(&format!("refs/remotes/origin/{}", target)))
                .map_err(|_| GitRepositoryError::BranchNotFound(target.to_string()))?;
            reference.peel_to_commit()?
        };
        
        let git2_reset_type = match reset_type {
            ResetMode::Soft => ResetType::Soft,
            ResetMode::Mixed => ResetType::Mixed,
            ResetMode::Hard => ResetType::Hard,
        };
        
        self.repo.reset(target_commit.as_object(), git2_reset_type, None)?;
        
        Ok(())
    }
    
    /// Fast-forward merge with upstream
    pub fn fast_forward_merge(&self, branch_name: &str) -> Result<(), GitRepositoryError> {
        // Get the current branch
        let head = self.repo.head()?;
        let head_commit = head.peel_to_commit()?;
        
        // Find the upstream branch
        let upstream_ref = format!("refs/remotes/origin/{}", branch_name);
        let upstream = self.repo.find_reference(&upstream_ref)
            .map_err(|_| GitRepositoryError::BranchNotFound(upstream_ref))?;
        let upstream_commit = upstream.peel_to_commit()?;
        
        // Check if fast-forward is possible
        let merge_base = self.repo.merge_base(head_commit.id(), upstream_commit.id())?;
        
        if merge_base != head_commit.id() {
            return Err(GitRepositoryError::MergeFailed(
                "Fast-forward merge not possible".to_string()
            ));
        }
        
        // Perform fast-forward
        let mut head_ref = self.repo.head()?;
        head_ref.set_target(upstream_commit.id(), "Fast-forward merge")?;
        
        // Update working directory
        let mut checkout_builder = CheckoutBuilder::new();
        checkout_builder.safe();
        self.repo.checkout_head(Some(&mut checkout_builder))?;
        
        Ok(())
    }
    
    /// Get repository status
    pub fn status(&self) -> Result<RepositoryStatus, GitRepositoryError> {
        // Get current branch
        let current_branch = self.get_current_branch()?;
        
        // Get current commit
        let head = self.repo.head()?;
        let current_commit = head.target().unwrap().to_string();
        
        // Check if working directory is clean
        let statuses = self.repo.statuses(None)?;
        let is_clean = statuses.is_empty();
        
        // Get modified, untracked, and staged files
        let mut modified_files = Vec::new();
        let mut untracked_files = Vec::new();
        let mut staged_files = Vec::new();
        
        for status in statuses.iter() {
            if let Some(path) = status.path() {
                let flags = status.status();
                if flags.is_wt_modified() {
                    modified_files.push(path.to_string());
                }
                if flags.is_wt_new() {
                    untracked_files.push(path.to_string());
                }
                if flags.is_index_modified() || flags.is_index_new() || flags.is_index_deleted() {
                    staged_files.push(path.to_string());
                }
            }
        }
        
        // Calculate ahead/behind (simplified - would need more complex logic for accurate count)
        let (ahead, behind) = self.calculate_ahead_behind(&current_branch)?;
        
        Ok(RepositoryStatus {
            current_branch: Some(current_branch),
            current_commit,
            is_clean,
            ahead,
            behind,
            modified_files,
            untracked_files,
            staged_files,
        })
    }
    
    /// Get current branch name
    pub fn get_current_branch(&self) -> Result<String, GitRepositoryError> {
        let head = self.repo.head()?;
        
        if let Some(branch_name) = head.shorthand() {
            Ok(branch_name.to_string())
        } else {
            // Detached HEAD
            let commit = head.target().unwrap();
            Ok(format!("detached@{}", commit.to_string()[..8].to_string()))
        }
    }
    
    /// List all branches
    pub fn list_branches(&self, branch_type: Option<GitBranchType>) -> Result<Vec<String>, GitRepositoryError> {
        let git2_branch_type = match branch_type {
            Some(GitBranchType::Local) => Some(BranchType::Local),
            Some(GitBranchType::Remote) => Some(BranchType::Remote),
            None => None,
        };
        
        let branches = self.repo.branches(git2_branch_type)?;
        let mut branch_names = Vec::new();
        
        for branch_result in branches {
            let (branch, _) = branch_result?;
            if let Some(name) = branch.name()? {
                branch_names.push(name.to_string());
            }
        }
        
        Ok(branch_names)
    }
    
    /// Check if working directory has uncommitted changes
    pub fn is_working_directory_clean(&self) -> Result<bool, GitRepositoryError> {
        let statuses = self.repo.statuses(None)?;
        Ok(statuses.is_empty())
    }
    
    /// Get repository path
    pub fn path(&self) -> &Path {
        &self.path
    }
    
    /// Get the underlying git2 repository
    pub fn git2_repo(&self) -> &Git2Repository {
        &self.repo
    }
    
    // Private helper methods
    
    /// Calculate ahead/behind commits for current branch
    fn calculate_ahead_behind(&self, branch_name: &str) -> Result<(usize, usize), GitRepositoryError> {
        // Simplified implementation - in a real scenario, you'd want more sophisticated logic
        let local_ref = format!("refs/heads/{}", branch_name);
        let remote_ref = format!("refs/remotes/origin/{}", branch_name);
        
        let local_commit = self.repo.find_reference(&local_ref)
            .and_then(|r| r.peel_to_commit())
            .map(|c| c.id());
        
        let remote_commit = self.repo.find_reference(&remote_ref)
            .and_then(|r| r.peel_to_commit())
            .map(|c| c.id());
        
        match (local_commit, remote_commit) {
            (Ok(local_oid), Ok(remote_oid)) => {
                if local_oid == remote_oid {
                    Ok((0, 0))
                } else {
                    // Simplified: just check if they're different
                    // TODO: Implement proper ahead/behind calculation using commit walking
                    Ok((1, 0)) // Placeholder
                }
            }
            _ => Ok((0, 0)),
        }
    }
}

/// Git reset modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResetMode {
    /// Soft reset - moves HEAD only
    Soft,
    /// Mixed reset - moves HEAD and resets index (default)
    Mixed,
    /// Hard reset - moves HEAD, resets index and working directory
    Hard,
}

/// Git branch types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GitBranchType {
    /// Local branches
    Local,
    /// Remote tracking branches
    Remote,
}

// TODO: Add comprehensive error recovery mechanisms
// TODO: Add support for Git LFS operations
// TODO: Add support for submodule operations
// TODO: Add support for Git hooks
// TODO: Add better progress reporting with structured data
// TODO: Add support for custom SSH keys and authentication methods
// TODO: Add support for Git worktrees
// TODO: Add better conflict resolution strategies
// TODO: Add support for Git attributes and gitignore handling
// TODO: Add comprehensive logging and tracing

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_repository_init() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().join("test_repo");
        
        let result = GitRepository::init(&repo_path, false);
        assert!(result.is_ok());
        
        let repo = result.unwrap();
        assert_eq!(repo.path(), repo_path);
    }
    
    #[test]
    fn test_repository_open_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let nonexistent_path = temp_dir.path().join("nonexistent");
        
        let result = GitRepository::open(&nonexistent_path);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), GitRepositoryError::RepositoryNotFound(_)));
    }
    
    #[test]
    fn test_clone_config_default() {
        let config = CloneConfig::default();
        assert!(config.branch.is_none());
        assert!(!config.shallow);
        assert!(config.depth.is_none());
        assert!(!config.recursive);
        assert!(config.progress_callback.is_none());
    }
    
    #[test]
    fn test_fetch_config_default() {
        let config = FetchConfig::default();
        assert_eq!(config.remote_name, "origin");
        assert!(config.refs.is_none());
        assert!(config.progress_callback.is_none());
    }
    
    #[test]
    fn test_reset_modes() {
        assert_eq!(ResetMode::Soft as u8, 0);
        assert_ne!(ResetMode::Soft, ResetMode::Mixed);
        assert_ne!(ResetMode::Mixed, ResetMode::Hard);
    }
    
    #[test]
    fn test_branch_types() {
        assert_ne!(GitBranchType::Local, GitBranchType::Remote);
    }
}