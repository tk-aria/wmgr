use thiserror::Error;
use git2::{Repository as Git2Repository, Direction};
use crate::domain::value_objects::{
    git_url::{GitUrl, GitUrlError},
};
use crate::domain::entities::repository::Remote as DomainRemote;

/// Git remote operations related errors
#[derive(Debug, Error)]
pub enum GitRemoteError {
    #[error("Remote not found: {0}")]
    RemoteNotFound(String),
    
    #[error("Remote already exists: {0}")]
    RemoteAlreadyExists(String),
    
    #[error("Invalid remote name: {0}")]
    InvalidRemoteName(String),
    
    #[error("Invalid remote URL: {0}")]
    InvalidRemoteUrl(String),
    
    #[error("Remote operation failed: {0}")]
    RemoteOperationFailed(String),
    
    #[error("Failed to add remote: {0}")]
    AddRemoteFailed(String),
    
    #[error("Failed to remove remote: {0}")]
    RemoveRemoteFailed(String),
    
    #[error("Failed to update remote URL: {0}")]
    UpdateRemoteUrlFailed(String),
    
    #[error("Failed to rename remote: {0}")]
    RenameRemoteFailed(String),
    
    #[error("Git URL error: {0}")]
    GitUrlError(#[from] GitUrlError),
    
    #[error("Git2 error: {0}")]
    Git2Error(#[from] git2::Error),
}

/// Remote information with additional metadata
#[derive(Debug, Clone)]
pub struct RemoteInfo {
    /// Remote name
    pub name: String,
    
    /// Remote URL
    pub url: GitUrl,
    
    /// Push URL (if different from fetch URL)
    pub push_url: Option<GitUrl>,
    
    /// Whether this remote is the default (origin)
    pub is_default: bool,
    
    /// List of fetch refspecs
    pub fetch_refspecs: Vec<String>,
    
    /// List of push refspecs
    pub push_refspecs: Vec<String>,
}

impl RemoteInfo {
    /// Create a new RemoteInfo
    pub fn new(name: String, url: GitUrl) -> Self {
        let is_default = name == "origin";
        
        Self {
            name,
            url,
            push_url: None,
            is_default,
            fetch_refspecs: Vec::new(),
            push_refspecs: Vec::new(),
        }
    }
    
    /// Set push URL (if different from fetch URL)
    pub fn with_push_url(mut self, push_url: GitUrl) -> Self {
        self.push_url = Some(push_url);
        self
    }
    
    /// Add fetch refspec
    pub fn with_fetch_refspec(mut self, refspec: String) -> Self {
        self.fetch_refspecs.push(refspec);
        self
    }
    
    /// Add push refspec
    pub fn with_push_refspec(mut self, refspec: String) -> Self {
        self.push_refspecs.push(refspec);
        self
    }
    
    /// Convert to domain Remote entity
    pub fn to_domain_remote(&self) -> DomainRemote {
        DomainRemote::new(&self.name, self.url.as_str())
    }
}

/// Git remote manager for handling remote operations
pub struct GitRemoteManager<'repo> {
    /// Reference to the git2 repository
    repo: &'repo Git2Repository,
}

impl<'repo> GitRemoteManager<'repo> {
    /// Create a new GitRemoteManager
    pub fn new(repo: &'repo Git2Repository) -> Self {
        Self { repo }
    }
    
    /// List all remotes
    pub fn list_remotes(&self) -> Result<Vec<RemoteInfo>, GitRemoteError> {
        let remote_names = self.repo.remotes()?;
        let mut remotes = Vec::new();
        
        for remote_name in remote_names.iter().flatten() {
            if let Ok(remote_info) = self.get_remote_info(remote_name) {
                remotes.push(remote_info);
            }
        }
        
        Ok(remotes)
    }
    
    /// Get information about a specific remote
    pub fn get_remote_info(&self, name: &str) -> Result<RemoteInfo, GitRemoteError> {
        let remote = self.repo.find_remote(name)
            .map_err(|_| GitRemoteError::RemoteNotFound(name.to_string()))?;
        
        // Get remote URL
        let url_str = remote.url()
            .ok_or_else(|| GitRemoteError::InvalidRemoteUrl("No URL found".to_string()))?;
        let url = GitUrl::new(url_str)?;
        
        // Get push URL if different
        let push_url = remote.pushurl()
            .map(|push_url_str| GitUrl::new(push_url_str))
            .transpose()?;
        
        // Get refspecs
        let fetch_refspecs: Vec<String> = remote.fetch_refspecs()
            .map(|refspecs| {
                (0..refspecs.len())
                    .filter_map(|i| refspecs.get(i).map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();
        
        let push_refspecs: Vec<String> = remote.push_refspecs()
            .map(|refspecs| {
                (0..refspecs.len())
                    .filter_map(|i| refspecs.get(i).map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();
        
        let mut remote_info = RemoteInfo::new(name.to_string(), url);
        
        if let Some(push_url) = push_url {
            remote_info = remote_info.with_push_url(push_url);
        }
        
        for refspec in fetch_refspecs {
            remote_info = remote_info.with_fetch_refspec(refspec);
        }
        
        for refspec in push_refspecs {
            remote_info = remote_info.with_push_refspec(refspec);
        }
        
        Ok(remote_info)
    }
    
    /// Add a new remote
    pub fn add_remote(&self, name: &str, url: &GitUrl) -> Result<RemoteInfo, GitRemoteError> {
        // Validate remote name
        self.validate_remote_name(name)?;
        
        // Check if remote already exists
        if self.remote_exists(name) {
            return Err(GitRemoteError::RemoteAlreadyExists(name.to_string()));
        }
        
        // Convert GitUrl to appropriate format for git2
        let url_str = self.get_optimal_url_format(url);
        
        // Add the remote
        let _remote = self.repo.remote(name, &url_str)
            .map_err(|e| GitRemoteError::AddRemoteFailed(e.to_string()))?;
        
        // Return the created remote info
        self.get_remote_info(name)
    }
    
    /// Remove a remote
    pub fn remove_remote(&self, name: &str) -> Result<(), GitRemoteError> {
        // Check if remote exists
        if !self.remote_exists(name) {
            return Err(GitRemoteError::RemoteNotFound(name.to_string()));
        }
        
        // Remove the remote
        self.repo.remote_delete(name)
            .map_err(|e| GitRemoteError::RemoveRemoteFailed(e.to_string()))?;
        
        Ok(())
    }
    
    /// Update remote URL
    pub fn set_remote_url(&self, name: &str, url: &GitUrl) -> Result<(), GitRemoteError> {
        // Check if remote exists
        if !self.remote_exists(name) {
            return Err(GitRemoteError::RemoteNotFound(name.to_string()));
        }
        
        // Convert GitUrl to appropriate format
        let url_str = self.get_optimal_url_format(url);
        
        // Update the URL
        self.repo.remote_set_url(name, &url_str)
            .map_err(|e| GitRemoteError::UpdateRemoteUrlFailed(e.to_string()))?;
        
        Ok(())
    }
    
    /// Update remote push URL
    pub fn set_remote_push_url(&self, name: &str, url: &GitUrl) -> Result<(), GitRemoteError> {
        // Check if remote exists
        if !self.remote_exists(name) {
            return Err(GitRemoteError::RemoteNotFound(name.to_string()));
        }
        
        // Convert GitUrl to appropriate format
        let url_str = self.get_optimal_url_format(url);
        
        // Update the push URL
        self.repo.remote_set_pushurl(name, Some(&url_str))
            .map_err(|e| GitRemoteError::UpdateRemoteUrlFailed(e.to_string()))?;
        
        Ok(())
    }
    
    /// Rename a remote
    pub fn rename_remote(&self, old_name: &str, new_name: &str) -> Result<(), GitRemoteError> {
        // Validate new remote name
        self.validate_remote_name(new_name)?;
        
        // Check if old remote exists
        if !self.remote_exists(old_name) {
            return Err(GitRemoteError::RemoteNotFound(old_name.to_string()));
        }
        
        // Check if new remote name is already taken
        if self.remote_exists(new_name) {
            return Err(GitRemoteError::RemoteAlreadyExists(new_name.to_string()));
        }
        
        // Rename the remote
        self.repo.remote_rename(old_name, new_name)
            .map_err(|e| GitRemoteError::RenameRemoteFailed(e.to_string()))?;
        
        Ok(())
    }
    
    /// Add or update a remote (convenience method)
    pub fn add_or_update_remote(&self, name: &str, url: &GitUrl) -> Result<RemoteInfo, GitRemoteError> {
        if self.remote_exists(name) {
            // Update existing remote
            self.set_remote_url(name, url)?;
            self.get_remote_info(name)
        } else {
            // Add new remote
            self.add_remote(name, url)
        }
    }
    
    /// Get the default remote (origin)
    pub fn get_default_remote(&self) -> Result<Option<RemoteInfo>, GitRemoteError> {
        if self.remote_exists("origin") {
            Ok(Some(self.get_remote_info("origin")?))
        } else {
            // If no origin, try to find the first remote
            let remotes = self.list_remotes()?;
            Ok(remotes.into_iter().next())
        }
    }
    
    /// Check if remote exists
    pub fn remote_exists(&self, name: &str) -> bool {
        self.repo.find_remote(name).is_ok()
    }
    
    /// Validate and normalize multiple remotes for a repository
    pub fn setup_remotes(&self, remotes: &[DomainRemote]) -> Result<Vec<RemoteInfo>, GitRemoteError> {
        let mut remote_infos = Vec::new();
        
        for domain_remote in remotes {
            let url = GitUrl::new(&domain_remote.url)?;
            let remote_info = self.add_or_update_remote(&domain_remote.name, &url)?;
            remote_infos.push(remote_info);
        }
        
        Ok(remote_infos)
    }
    
    /// Get remote URL for a specific operation (fetch/push)
    pub fn get_remote_url(&self, name: &str, for_push: bool) -> Result<GitUrl, GitRemoteError> {
        let remote_info = self.get_remote_info(name)?;
        
        if for_push && remote_info.push_url.is_some() {
            Ok(remote_info.push_url.unwrap())
        } else {
            Ok(remote_info.url)
        }
    }
    
    /// Prune remote tracking branches
    pub fn prune_remote(&self, name: &str) -> Result<Vec<String>, GitRemoteError> {
        let mut remote = self.repo.find_remote(name)
            .map_err(|_| GitRemoteError::RemoteNotFound(name.to_string()))?;
        
        // Get remote heads to determine what should be pruned
        let mut callbacks = git2::RemoteCallbacks::new();
        
        // Set up authentication (simplified)
        callbacks.credentials(|_url, username_from_url, allowed_types| {
            if allowed_types.contains(git2::CredentialType::SSH_KEY) {
                git2::Cred::ssh_key_from_agent(username_from_url.unwrap_or("git"))
            } else {
                Err(git2::Error::from_str("Authentication not supported"))
            }
        });
        
        // Connect to remote to get latest refs
        remote.connect_auth(Direction::Fetch, Some(callbacks), None)?;
        
        let remote_refs: Vec<_> = remote.list()?.iter()
            .map(|head| head.name().to_string())
            .collect();
        
        remote.disconnect()?;
        
        // Find local tracking branches that no longer exist on remote
        let local_branches = self.repo.branches(Some(git2::BranchType::Remote))?;
        let mut pruned_branches = Vec::new();
        
        for branch_result in local_branches {
            let (branch, _) = branch_result?;
            if let Some(branch_name) = branch.name()? {
                if branch_name.starts_with(&format!("{}/", name)) {
                    let remote_ref = format!("refs/heads/{}", 
                        branch_name.strip_prefix(&format!("{}/", name)).unwrap());
                    
                    if !remote_refs.contains(&remote_ref) {
                        // This branch should be pruned
                        let full_branch_name = format!("refs/remotes/{}", branch_name);
                        self.repo.find_reference(&full_branch_name)?.delete()?;
                        pruned_branches.push(branch_name.to_string());
                    }
                }
            }
        }
        
        Ok(pruned_branches)
    }
    
    // Private helper methods
    
    /// Validate remote name according to Git rules
    fn validate_remote_name(&self, name: &str) -> Result<(), GitRemoteError> {
        if name.is_empty() {
            return Err(GitRemoteError::InvalidRemoteName("Remote name cannot be empty".to_string()));
        }
        
        // Git remote name rules:
        // - Cannot start with '.'
        // - Cannot contain certain characters
        if name.starts_with('.') {
            return Err(GitRemoteError::InvalidRemoteName(
                "Remote name cannot start with '.'".to_string()
            ));
        }
        
        // Check for invalid characters
        let invalid_chars = [' ', '\t', '\n', '\r', ':', '?', '*', '[', '\\', '^', '~'];
        if name.chars().any(|c| invalid_chars.contains(&c) || c.is_ascii_control()) {
            return Err(GitRemoteError::InvalidRemoteName(
                format!("Remote name contains invalid characters: {}", name)
            ));
        }
        
        // Cannot be "HEAD"
        if name == "HEAD" {
            return Err(GitRemoteError::InvalidRemoteName(
                "Remote name cannot be 'HEAD'".to_string()
            ));
        }
        
        Ok(())
    }
    
    /// Get optimal URL format based on GitUrl capabilities
    fn get_optimal_url_format(&self, url: &GitUrl) -> String {
        // For most cases, use the normalized HTTPS URL
        // In production, you might want to prefer SSH for authenticated operations
        url.to_https_url()
    }
}

/// Utility functions for remote operations
pub mod utils {
    use super::*;
    
    /// Normalize remote name (ensure it follows Git conventions)
    pub fn normalize_remote_name(name: &str) -> String {
        // Basic normalization - replace invalid characters with underscores
        name.chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect::<String>()
            .trim_start_matches('.')
            .to_string()
    }
    
    /// Extract organization and repository name from URL
    pub fn extract_repo_info(url: &GitUrl) -> Option<(String, String)> {
        if let (Some(org), Some(repo)) = (url.organization(), url.repo_name()) {
            Some((org.to_string(), repo.to_string()))
        } else {
            None
        }
    }
    
    /// Generate remote name from URL (useful for auto-naming)
    pub fn suggest_remote_name(url: &GitUrl, existing_remotes: &[String]) -> String {
        if let Some((org, _repo)) = extract_repo_info(url) {
            let base_name = normalize_remote_name(&org);
            
            if !existing_remotes.contains(&base_name) {
                base_name
            } else {
                // Add suffix if name already exists
                for i in 1..100 {
                    let candidate = format!("{}{}", base_name, i);
                    if !existing_remotes.contains(&candidate) {
                        return candidate;
                    }
                }
                format!("remote{}", existing_remotes.len())
            }
        } else {
            "remote".to_string()
        }
    }
    
    /// Check if two URLs point to the same repository
    pub fn are_same_repository(url1: &GitUrl, url2: &GitUrl) -> bool {
        url1.is_same_repo(url2)
    }
}

// TODO: Add support for remote authentication configuration
// TODO: Add support for multiple push URLs per remote
// TODO: Add support for custom refspecs management
// TODO: Add support for remote pruning strategies
// TODO: Add better error recovery for network operations
// TODO: Add support for remote repository validation
// TODO: Add support for remote mirroring operations
// TODO: Add comprehensive logging for remote operations
// TODO: Add support for remote-specific configuration
// TODO: Add integration with credential managers

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use git2::Repository as Git2Repository;
    
    fn create_test_repo() -> (TempDir, Git2Repository) {
        let temp_dir = TempDir::new().unwrap();
        let repo = Git2Repository::init(temp_dir.path()).unwrap();
        (temp_dir, repo)
    }
    
    #[test]
    fn test_remote_info_creation() {
        let url = GitUrl::new("https://github.com/example/repo.git").unwrap();
        let remote_info = RemoteInfo::new("origin".to_string(), url.clone());
        
        assert_eq!(remote_info.name, "origin");
        assert_eq!(remote_info.url, url);
        assert!(remote_info.is_default);
        assert!(remote_info.push_url.is_none());
    }
    
    #[test]
    fn test_remote_info_builder() {
        let url = GitUrl::new("https://github.com/example/repo.git").unwrap();
        let push_url = GitUrl::new("git@github.com:example/repo.git").unwrap();
        
        let remote_info = RemoteInfo::new("upstream".to_string(), url.clone())
            .with_push_url(push_url.clone())
            .with_fetch_refspec("+refs/heads/*:refs/remotes/upstream/*".to_string())
            .with_push_refspec("refs/heads/*:refs/heads/*".to_string());
        
        assert_eq!(remote_info.name, "upstream");
        assert!(!remote_info.is_default);
        assert_eq!(remote_info.push_url, Some(push_url));
        assert_eq!(remote_info.fetch_refspecs.len(), 1);
        assert_eq!(remote_info.push_refspecs.len(), 1);
    }
    
    #[test]
    fn test_remote_manager_creation() {
        let (_temp_dir, repo) = create_test_repo();
        let manager = GitRemoteManager::new(&repo);
        
        let remotes = manager.list_remotes().unwrap();
        assert!(remotes.is_empty());
    }
    
    #[test]
    fn test_add_remote() {
        let (_temp_dir, repo) = create_test_repo();
        let manager = GitRemoteManager::new(&repo);
        let url = GitUrl::new("https://github.com/example/repo.git").unwrap();
        
        let result = manager.add_remote("origin", &url);
        assert!(result.is_ok());
        
        let remote_info = result.unwrap();
        assert_eq!(remote_info.name, "origin");
        assert!(remote_info.is_default);
    }
    
    #[test]
    fn test_add_duplicate_remote() {
        let (_temp_dir, repo) = create_test_repo();
        let manager = GitRemoteManager::new(&repo);
        let url = GitUrl::new("https://github.com/example/repo.git").unwrap();
        
        // Add first remote
        manager.add_remote("origin", &url).unwrap();
        
        // Try to add same remote again
        let result = manager.add_remote("origin", &url);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), GitRemoteError::RemoteAlreadyExists(_)));
    }
    
    #[test]
    fn test_validate_remote_name() {
        let (_temp_dir, repo) = create_test_repo();
        let manager = GitRemoteManager::new(&repo);
        
        // Valid names
        assert!(manager.validate_remote_name("origin").is_ok());
        assert!(manager.validate_remote_name("upstream").is_ok());
        assert!(manager.validate_remote_name("remote-1").is_ok());
        
        // Invalid names
        assert!(manager.validate_remote_name("").is_err());
        assert!(manager.validate_remote_name(".hidden").is_err());
        assert!(manager.validate_remote_name("HEAD").is_err());
        assert!(manager.validate_remote_name("remote with spaces").is_err());
    }
    
    #[test]
    fn test_utils_normalize_remote_name() {
        assert_eq!(utils::normalize_remote_name("origin"), "origin");
        assert_eq!(utils::normalize_remote_name("remote with spaces"), "remote_with_spaces");
        assert_eq!(utils::normalize_remote_name(".hidden"), "_hidden");
        assert_eq!(utils::normalize_remote_name("remote:with:colons"), "remote_with_colons");
    }
    
    #[test]
    fn test_utils_extract_repo_info() {
        let url = GitUrl::new("https://github.com/example/repo.git").unwrap();
        let (org, repo) = utils::extract_repo_info(&url).unwrap();
        
        assert_eq!(org, "example");
        assert_eq!(repo, "repo");
    }
    
    #[test]
    fn test_utils_suggest_remote_name() {
        let url = GitUrl::new("https://github.com/example/repo.git").unwrap();
        let existing = vec![];
        
        let suggested = utils::suggest_remote_name(&url, &existing);
        assert_eq!(suggested, "example");
        
        let existing = vec!["example".to_string()];
        let suggested = utils::suggest_remote_name(&url, &existing);
        assert_eq!(suggested, "example1");
    }
    
    #[test]
    fn test_domain_remote_conversion() {
        let url = GitUrl::new("https://github.com/example/repo.git").unwrap();
        let remote_info = RemoteInfo::new("origin".to_string(), url.clone());
        let domain_remote = remote_info.to_domain_remote();
        
        assert_eq!(domain_remote.name, "origin");
        assert_eq!(domain_remote.url, url.as_str());
    }
}