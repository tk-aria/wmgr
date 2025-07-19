//! Mock services for testing
//!
//! This module provides mock implementations of services and repositories
//! that can be used in tests to isolate units under test.

use async_trait::async_trait;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tsrc::application::services::manifest_service::{
    ManifestService, ManifestServiceError, ProcessedManifest,
};
use tsrc::domain::{
    entities::{manifest::Manifest, repository::Repository, workspace::Workspace},
    value_objects::git_url::GitUrl,
};
// Note: These would be actual imports in a real implementation
// use tsrc::infrastructure::git::repository::{GitRepository, CloneConfig, FetchConfig, ResetMode};

// For now, we'll create mock types
pub struct CloneConfig;
pub struct FetchConfig;
#[derive(Debug, Clone, Copy)]
pub enum ResetMode {
    Soft,
    Mixed,
    Hard,
}

impl Default for CloneConfig {
    fn default() -> Self {
        Self
    }
}

impl Default for FetchConfig {
    fn default() -> Self {
        Self
    }
}

/// Mock manifest service for testing
pub struct MockManifestService {
    /// Stored manifests indexed by file path or URL
    manifests: Arc<Mutex<HashMap<String, Manifest>>>,
    /// Call history for verification
    call_history: Arc<Mutex<Vec<String>>>,
    /// Whether to simulate errors
    should_error: Arc<Mutex<bool>>,
}

impl MockManifestService {
    /// Create a new mock manifest service
    pub fn new() -> Self {
        Self {
            manifests: Arc::new(Mutex::new(HashMap::new())),
            call_history: Arc::new(Mutex::new(Vec::new())),
            should_error: Arc::new(Mutex::new(false)),
        }
    }

    /// Add a manifest to the mock service
    pub fn add_manifest(&self, path_or_url: &str, manifest: Manifest) {
        let mut manifests = self.manifests.lock().unwrap();
        manifests.insert(path_or_url.to_string(), manifest);
    }

    /// Get call history for verification
    pub fn get_call_history(&self) -> Vec<String> {
        let history = self.call_history.lock().unwrap();
        history.clone()
    }

    /// Clear call history
    pub fn clear_call_history(&self) {
        let mut history = self.call_history.lock().unwrap();
        history.clear();
    }

    /// Set whether the service should simulate errors
    pub fn set_should_error(&self, should_error: bool) {
        let mut error_flag = self.should_error.lock().unwrap();
        *error_flag = should_error;
    }

    /// Record a method call
    fn record_call(&self, method: &str, arg: &str) {
        let mut history = self.call_history.lock().unwrap();
        history.push(format!("{}({})", method, arg));
    }

    /// Check if should return error
    fn should_return_error(&self) -> bool {
        let error_flag = self.should_error.lock().unwrap();
        *error_flag
    }
}

impl Default for MockManifestService {
    fn default() -> Self {
        Self::new()
    }
}

// Note: This would implement the actual ManifestService trait if it was defined as a trait
// For now, we'll create methods that match the expected interface

impl MockManifestService {
    /// Mock parse_from_file method
    pub async fn parse_from_file(
        &self,
        path: &Path,
    ) -> Result<ProcessedManifest, ManifestServiceError> {
        let path_str = path.to_string_lossy().to_string();
        self.record_call("parse_from_file", &path_str);

        if self.should_return_error() {
            return Err(ManifestServiceError::ParseError("Mock error".to_string()));
        }

        let manifests = self.manifests.lock().unwrap();
        if let Some(manifest) = manifests.get(&path_str) {
            Ok(ProcessedManifest {
                manifest: manifest.clone(),
                warnings: vec![],
                includes: vec![],
            })
        } else {
            Err(ManifestServiceError::ParseError(format!(
                "Manifest not found: {}",
                path_str
            )))
        }
    }

    /// Mock parse_from_url method
    pub async fn parse_from_url(
        &self,
        url: &str,
    ) -> Result<ProcessedManifest, ManifestServiceError> {
        self.record_call("parse_from_url", url);

        if self.should_return_error() {
            return Err(ManifestServiceError::RemoteManifestFetchFailed {
                url: url.to_string(),
                reason: "Mock HTTP error".to_string(),
            });
        }

        let manifests = self.manifests.lock().unwrap();
        if let Some(manifest) = manifests.get(url) {
            Ok(ProcessedManifest {
                manifest: manifest.clone(),
                warnings: vec![],
                includes: vec![],
            })
        } else {
            Err(ManifestServiceError::RemoteManifestFetchFailed {
                url: url.to_string(),
                reason: "Not found in mock".to_string(),
            })
        }
    }

    /// Mock validate_manifest method
    pub fn validate_manifest(&self, manifest: &Manifest) -> Result<(), ManifestServiceError> {
        self.record_call(
            "validate_manifest",
            &format!("{} repos", manifest.repos.len()),
        );

        if self.should_return_error() {
            return Err(ManifestServiceError::ValidationError(
                "Mock validation error".to_string(),
            ));
        }

        // Simple validation - check for duplicate destinations
        let mut destinations = std::collections::HashSet::new();
        for repo in &manifest.repos {
            if !destinations.insert(&repo.dest) {
                return Err(ManifestServiceError::ValidationError(format!(
                    "Duplicate destination: {}",
                    repo.dest
                )));
            }
        }

        Ok(())
    }
}

/// Mock Git repository for testing
pub struct MockGitRepository {
    /// Repository path
    path: PathBuf,
    /// Current branch
    current_branch: String,
    /// Available branches
    branches: Vec<String>,
    /// Remotes
    remotes: HashMap<String, String>,
    /// Whether operations should fail
    should_fail: Arc<Mutex<bool>>,
    /// Call history
    call_history: Arc<Mutex<Vec<String>>>,
}

impl MockGitRepository {
    /// Create a new mock Git repository
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            current_branch: "main".to_string(),
            branches: vec!["main".to_string()],
            remotes: HashMap::new(),
            should_fail: Arc::new(Mutex::new(false)),
            call_history: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Set whether operations should fail
    pub fn set_should_fail(&self, should_fail: bool) {
        let mut fail_flag = self.should_fail.lock().unwrap();
        *fail_flag = should_fail;
    }

    /// Get call history
    pub fn get_call_history(&self) -> Vec<String> {
        let history = self.call_history.lock().unwrap();
        history.clone()
    }

    /// Record a method call
    fn record_call(&self, method: &str) {
        let mut history = self.call_history.lock().unwrap();
        history.push(method.to_string());
    }

    /// Check if should fail
    fn should_return_error(&self) -> bool {
        let fail_flag = self.should_fail.lock().unwrap();
        *fail_flag
    }

    /// Add a branch
    pub fn add_branch(&mut self, branch_name: &str) {
        self.branches.push(branch_name.to_string());
    }

    /// Add a remote
    pub fn add_remote(&mut self, name: &str, url: &str) {
        self.remotes.insert(name.to_string(), url.to_string());
    }

    /// Set current branch
    pub fn set_current_branch(&mut self, branch: &str) {
        self.current_branch = branch.to_string();
        if !self.branches.contains(&branch.to_string()) {
            self.branches.push(branch.to_string());
        }
    }
}

// Mock implementations of GitRepository methods
impl MockGitRepository {
    /// Mock init method
    pub fn init_mock(path: &Path) -> Result<Self, std::io::Error> {
        Ok(Self::new(path.to_path_buf()))
    }

    /// Mock open method
    pub fn open_mock(path: &Path) -> Result<Self, std::io::Error> {
        if !path.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Repository not found",
            ));
        }
        Ok(Self::new(path.to_path_buf()))
    }

    /// Mock clone method
    pub async fn clone_mock(
        url: &GitUrl,
        path: &Path,
        _config: &CloneConfig,
    ) -> Result<Self, std::io::Error> {
        let mut repo = Self::new(path.to_path_buf());
        repo.record_call("clone");

        if repo.should_return_error() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Mock clone failed",
            ));
        }

        repo.add_remote("origin", url.as_str());
        Ok(repo)
    }

    /// Mock current_branch method
    pub fn current_branch(&self) -> Result<String, std::io::Error> {
        self.record_call("current_branch");

        if self.should_return_error() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Mock current_branch failed",
            ));
        }

        Ok(self.current_branch.clone())
    }

    /// Mock fetch method
    pub async fn fetch(&self, _remote: &str, _config: &FetchConfig) -> Result<(), std::io::Error> {
        self.record_call("fetch");

        if self.should_return_error() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Mock fetch failed",
            ));
        }

        Ok(())
    }

    /// Mock reset method
    pub fn reset(&self, _target: &str, _mode: ResetMode) -> Result<(), std::io::Error> {
        self.record_call("reset");

        if self.should_return_error() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Mock reset failed",
            ));
        }

        Ok(())
    }

    /// Mock is_valid method
    pub fn is_valid(&self) -> bool {
        self.record_call("is_valid");
        !self.should_return_error()
    }
}

/// Mock command executor for testing
pub struct MockCommandExecutor {
    /// Command execution results
    results: Arc<Mutex<HashMap<String, (i32, String, String)>>>, // (exit_code, stdout, stderr)
    /// Call history
    call_history: Arc<Mutex<Vec<String>>>,
    /// Whether to simulate failures
    should_fail: Arc<Mutex<bool>>,
}

impl MockCommandExecutor {
    /// Create a new mock command executor
    pub fn new() -> Self {
        Self {
            results: Arc::new(Mutex::new(HashMap::new())),
            call_history: Arc::new(Mutex::new(Vec::new())),
            should_fail: Arc::new(Mutex::new(false)),
        }
    }

    /// Set the result for a specific command
    pub fn set_command_result(&self, command: &str, exit_code: i32, stdout: &str, stderr: &str) {
        let mut results = self.results.lock().unwrap();
        results.insert(
            command.to_string(),
            (exit_code, stdout.to_string(), stderr.to_string()),
        );
    }

    /// Get call history
    pub fn get_call_history(&self) -> Vec<String> {
        let history = self.call_history.lock().unwrap();
        history.clone()
    }

    /// Set whether to simulate failures
    pub fn set_should_fail(&self, should_fail: bool) {
        let mut fail_flag = self.should_fail.lock().unwrap();
        *fail_flag = should_fail;
    }

    /// Record a method call
    fn record_call(&self, command: &str) {
        let mut history = self.call_history.lock().unwrap();
        history.push(command.to_string());
    }

    /// Mock execute method
    pub async fn execute(&self, command: &str) -> Result<(i32, String, String), std::io::Error> {
        self.record_call(command);

        if *self.should_fail.lock().unwrap() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Mock command execution failed",
            ));
        }

        let results = self.results.lock().unwrap();
        if let Some((exit_code, stdout, stderr)) = results.get(command) {
            Ok((*exit_code, stdout.clone(), stderr.clone()))
        } else {
            // Default success result
            Ok((0, format!("Mock output for: {}", command), String::new()))
        }
    }
}

impl Default for MockCommandExecutor {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock file system operations
pub struct MockFileSystem {
    /// Files stored in memory
    files: Arc<Mutex<HashMap<PathBuf, String>>>,
    /// Directories
    directories: Arc<Mutex<std::collections::HashSet<PathBuf>>>,
    /// Call history
    call_history: Arc<Mutex<Vec<String>>>,
    /// Whether operations should fail
    should_fail: Arc<Mutex<bool>>,
}

impl MockFileSystem {
    /// Create a new mock file system
    pub fn new() -> Self {
        Self {
            files: Arc::new(Mutex::new(HashMap::new())),
            directories: Arc::new(Mutex::new(std::collections::HashSet::new())),
            call_history: Arc::new(Mutex::new(Vec::new())),
            should_fail: Arc::new(Mutex::new(false)),
        }
    }

    /// Add a file
    pub fn add_file(&self, path: &Path, content: &str) {
        let mut files = self.files.lock().unwrap();
        files.insert(path.to_path_buf(), content.to_string());

        // Also add parent directories
        if let Some(parent) = path.parent() {
            let mut directories = self.directories.lock().unwrap();
            directories.insert(parent.to_path_buf());
        }
    }

    /// Add a directory
    pub fn add_directory(&self, path: &Path) {
        let mut directories = self.directories.lock().unwrap();
        directories.insert(path.to_path_buf());
    }

    /// Check if file exists
    pub fn file_exists(&self, path: &Path) -> bool {
        let files = self.files.lock().unwrap();
        files.contains_key(path)
    }

    /// Check if directory exists
    pub fn directory_exists(&self, path: &Path) -> bool {
        let directories = self.directories.lock().unwrap();
        directories.contains(path)
    }

    /// Read file content
    pub fn read_file(&self, path: &Path) -> Result<String, std::io::Error> {
        self.record_call(&format!("read_file({})", path.display()));

        if *self.should_fail.lock().unwrap() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Mock file read failed",
            ));
        }

        let files = self.files.lock().unwrap();
        if let Some(content) = files.get(path) {
            Ok(content.clone())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "File not found in mock filesystem",
            ))
        }
    }

    /// Write file content
    pub fn write_file(&self, path: &Path, content: &str) -> Result<(), std::io::Error> {
        self.record_call(&format!("write_file({})", path.display()));

        if *self.should_fail.lock().unwrap() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Mock file write failed",
            ));
        }

        self.add_file(path, content);
        Ok(())
    }

    /// Get call history
    pub fn get_call_history(&self) -> Vec<String> {
        let history = self.call_history.lock().unwrap();
        history.clone()
    }

    /// Set whether operations should fail
    pub fn set_should_fail(&self, should_fail: bool) {
        let mut fail_flag = self.should_fail.lock().unwrap();
        *fail_flag = should_fail;
    }

    /// Record a method call
    fn record_call(&self, operation: &str) {
        let mut history = self.call_history.lock().unwrap();
        history.push(operation.to_string());
    }
}

impl Default for MockFileSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::test_fixtures::ManifestFixture;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_mock_manifest_service() {
        let mock_service = MockManifestService::new();
        let manifest = ManifestFixture::simple();

        // Add manifest to mock
        mock_service.add_manifest("/test/manifest.yml", manifest.clone());

        // Test parse_from_file
        let path = Path::new("/test/manifest.yml");
        let result = mock_service.parse_from_file(path).await;
        assert!(result.is_ok());

        let processed = result.unwrap();
        assert_eq!(processed.manifest.repos.len(), manifest.repos.len());

        // Check call history
        let history = mock_service.get_call_history();
        assert_eq!(history.len(), 1);
        assert!(history[0].contains("parse_from_file"));
    }

    #[tokio::test]
    async fn test_mock_manifest_service_error() {
        let mock_service = MockManifestService::new();
        mock_service.set_should_error(true);

        let path = Path::new("/nonexistent/manifest.yml");
        let result = mock_service.parse_from_file(path).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_mock_git_repository() {
        let temp_dir = TempDir::new().unwrap();
        let mut mock_repo = MockGitRepository::new(temp_dir.path().to_path_buf());

        // Test basic operations
        assert_eq!(mock_repo.current_branch().unwrap(), "main");
        assert!(mock_repo.is_valid());

        // Add branches and remotes
        mock_repo.add_branch("develop");
        mock_repo.add_remote("origin", "https://github.com/example/repo.git");
        mock_repo.set_current_branch("develop");

        assert_eq!(mock_repo.current_branch().unwrap(), "develop");
        assert_eq!(mock_repo.branches.len(), 2);
        assert_eq!(mock_repo.remotes.len(), 1);
    }

    #[tokio::test]
    async fn test_mock_command_executor() {
        let mock_executor = MockCommandExecutor::new();

        // Set up expected result
        mock_executor.set_command_result("echo hello", 0, "hello\n", "");

        // Execute command
        let result = mock_executor.execute("echo hello").await;
        assert!(result.is_ok());

        let (exit_code, stdout, stderr) = result.unwrap();
        assert_eq!(exit_code, 0);
        assert_eq!(stdout, "hello\n");
        assert_eq!(stderr, "");

        // Check call history
        let history = mock_executor.get_call_history();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0], "echo hello");
    }

    #[test]
    fn test_mock_file_system() {
        let mock_fs = MockFileSystem::new();
        let file_path = Path::new("/test/file.txt");

        // Add file
        mock_fs.add_file(file_path, "test content");

        // Check existence
        assert!(mock_fs.file_exists(file_path));

        // Read content
        let content = mock_fs.read_file(file_path).unwrap();
        assert_eq!(content, "test content");

        // Write content
        mock_fs.write_file(file_path, "new content").unwrap();
        let content = mock_fs.read_file(file_path).unwrap();
        assert_eq!(content, "new content");
    }
}
