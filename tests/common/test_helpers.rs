//! Test helper functions and utilities
//!
//! This module provides common helper functions that can be used across
//! different tests to perform common operations and validations.

use std::path::{Path, PathBuf};
use std::time::Duration;
use tempfile::TempDir;
use tokio::time::sleep;
use tsrc::domain::{
    entities::{manifest::Manifest, workspace::Workspace},
    value_objects::git_url::GitUrl,
};

/// Helper functions for file system operations in tests
pub struct FileSystemHelper;

impl FileSystemHelper {
    /// Wait for a file to exist (useful for async operations)
    pub async fn wait_for_file(path: &Path, timeout: Duration) -> bool {
        let start = std::time::Instant::now();

        while start.elapsed() < timeout {
            if path.exists() {
                return true;
            }
            sleep(Duration::from_millis(10)).await;
        }

        false
    }

    /// Wait for a file to be deleted
    pub async fn wait_for_file_deletion(path: &Path, timeout: Duration) -> bool {
        let start = std::time::Instant::now();

        while start.elapsed() < timeout {
            if !path.exists() {
                return true;
            }
            sleep(Duration::from_millis(10)).await;
        }

        false
    }

    /// Create a temporary file with specified content
    pub fn create_temp_file(dir: &TempDir, filename: &str, content: &str) -> PathBuf {
        let file_path = dir.path().join(filename);
        std::fs::write(&file_path, content).expect("Failed to write temp file");
        file_path
    }

    /// Read file content and return as string
    pub fn read_file_content(path: &Path) -> std::io::Result<String> {
        std::fs::read_to_string(path)
    }

    /// Check if directory structure matches expected pattern
    pub fn verify_directory_structure(base_path: &Path, expected_paths: &[&str]) -> bool {
        for expected_path in expected_paths {
            let full_path = base_path.join(expected_path);
            if !full_path.exists() {
                eprintln!("Expected path does not exist: {}", full_path.display());
                return false;
            }
        }
        true
    }

    /// Get file size in bytes
    pub fn get_file_size(path: &Path) -> std::io::Result<u64> {
        let metadata = std::fs::metadata(path)?;
        Ok(metadata.len())
    }

    /// Count files in directory (non-recursive)
    pub fn count_files_in_directory(dir: &Path) -> std::io::Result<usize> {
        let mut count = 0;
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                count += 1;
            }
        }
        Ok(count)
    }
}

/// Helper functions for workspace operations in tests
pub struct WorkspaceHelper;

impl WorkspaceHelper {
    /// Set up a complete test workspace with directories and files
    pub fn setup_test_workspace(temp_dir: &TempDir, workspace: &Workspace) -> std::io::Result<()> {
        let workspace_path = workspace.root_path.as_path();

        // Create .tsrc directory
        let tsrc_dir = workspace.tsrc_dir();
        std::fs::create_dir_all(&tsrc_dir)?;

        // Create config.yml
        let config_content = format!(
            r#"manifest_url: {}
manifest_branch: {}
groups: []
shallow: false
"#,
            workspace.config.manifest_url, workspace.config.manifest_branch
        );
        std::fs::write(workspace.config_path(), config_content)?;

        // Create manifest.yml if manifest exists
        if let Some(manifest) = &workspace.manifest {
            let manifest_content = Self::serialize_manifest_to_yaml(manifest)?;
            std::fs::write(workspace.manifest_file_path(), manifest_content)?;
        }

        // Create repository directories if manifest exists
        if let Some(manifest) = &workspace.manifest {
            for repo in &manifest.repos {
                let repo_path = workspace.repo_path(&repo.dest);
                std::fs::create_dir_all(&repo_path)?;

                // Create basic files in each repository
                std::fs::write(repo_path.join("README.md"), format!("# {}\n", repo.dest))?;
                std::fs::write(repo_path.join(".gitignore"), "target/\n*.log\n")?;
            }
        }

        Ok(())
    }

    /// Verify workspace structure is correct
    pub fn verify_workspace_structure(workspace: &Workspace) -> bool {
        let required_paths = vec![workspace.tsrc_dir(), workspace.config_path()];

        for path in required_paths {
            if !path.exists() {
                eprintln!("Required workspace path does not exist: {}", path.display());
                return false;
            }
        }

        // Check manifest file exists if workspace has manifest
        if workspace.manifest.is_some() {
            let manifest_path = workspace.manifest_file_path();
            if !manifest_path.exists() {
                eprintln!("Manifest file does not exist: {}", manifest_path.display());
                return false;
            }
        }

        true
    }

    /// Clean up workspace (remove all files)
    pub fn cleanup_workspace(workspace: &Workspace) -> std::io::Result<()> {
        if workspace.root_path.exists() {
            std::fs::remove_dir_all(&workspace.root_path)?;
        }
        Ok(())
    }

    /// Get repository paths that exist in the workspace
    pub fn get_existing_repository_paths(workspace: &Workspace) -> Vec<PathBuf> {
        let mut paths = Vec::new();

        if let Some(manifest) = &workspace.manifest {
            for repo in &manifest.repos {
                let repo_path = workspace.repo_path(&repo.dest);
                if repo_path.exists() {
                    paths.push(repo_path);
                }
            }
        }

        paths
    }

    /// Simple YAML serialization for manifest (for testing)
    fn serialize_manifest_to_yaml(manifest: &Manifest) -> std::io::Result<String> {
        let mut yaml = String::from("repos:\n");

        for repo in &manifest.repos {
            yaml.push_str(&format!("  - dest: {}\n", repo.dest));
            yaml.push_str(&format!("    url: {}\n", repo.url));
            if let Some(branch) = &repo.branch {
                yaml.push_str(&format!("    branch: {}\n", branch));
            }
        }

        if let Some(groups) = &manifest.groups {
            yaml.push_str("\ngroups:\n");
            for (group_name, group) in groups {
                yaml.push_str(&format!("  {}:\n", group_name));
                yaml.push_str("    repos:\n");
                for repo_dest in &group.repos {
                    yaml.push_str(&format!("      - {}\n", repo_dest));
                }
                if let Some(description) = &group.description {
                    yaml.push_str(&format!("    description: \"{}\"\n", description));
                }
            }
        }

        if let Some(default_branch) = &manifest.default_branch {
            yaml.push_str(&format!("\ndefault_branch: {}\n", default_branch));
        }

        Ok(yaml)
    }
}

/// Helper functions for Git URL operations in tests
pub struct GitUrlHelper;

impl GitUrlHelper {
    /// Convert between HTTPS and SSH formats
    pub fn convert_url_formats(url: &GitUrl) -> (String, String) {
        (url.to_https_url(), url.to_ssh_url())
    }

    /// Check if two URLs represent the same repository
    pub fn urls_are_same_repo(url1: &GitUrl, url2: &GitUrl) -> bool {
        url1.is_same_repo(url2)
    }

    /// Extract repository information from URLs
    pub fn extract_repo_info(url: &GitUrl) -> (Option<String>, Option<String>) {
        (url.organization(), url.repo_name())
    }

    /// Create test URLs with common patterns
    pub fn create_test_urls(base_org: &str, base_repo: &str) -> Vec<GitUrl> {
        vec![
            GitUrl::new(&format!(
                "https://github.com/{}/{}.git",
                base_org, base_repo
            ))
            .unwrap(),
            GitUrl::new(&format!("git@github.com:{}/{}.git", base_org, base_repo)).unwrap(),
            GitUrl::new(&format!(
                "https://gitlab.com/{}/{}.git",
                base_org, base_repo
            ))
            .unwrap(),
            GitUrl::new(&format!("git@gitlab.com:{}/{}.git", base_org, base_repo)).unwrap(),
        ]
    }
}

/// Helper functions for timing and async operations
pub struct AsyncHelper;

impl AsyncHelper {
    /// Run a test with timeout
    pub async fn with_timeout<F, T>(
        future: F,
        timeout: Duration,
    ) -> Result<T, tokio::time::error::Elapsed>
    where
        F: std::future::Future<Output = T>,
    {
        tokio::time::timeout(timeout, future).await
    }

    /// Wait for a condition to become true
    pub async fn wait_for_condition<F>(
        mut condition: F,
        timeout: Duration,
        check_interval: Duration,
    ) -> bool
    where
        F: FnMut() -> bool,
    {
        let start = std::time::Instant::now();

        while start.elapsed() < timeout {
            if condition() {
                return true;
            }
            sleep(check_interval).await;
        }

        false
    }

    /// Retry an async operation with exponential backoff
    pub async fn retry_with_backoff<F, T, E>(
        mut operation: F,
        max_retries: usize,
        initial_delay: Duration,
    ) -> Result<T, E>
    where
        F: FnMut() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>>>>,
    {
        let mut delay = initial_delay;

        for attempt in 0..max_retries {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(err) => {
                    if attempt == max_retries - 1 {
                        return Err(err);
                    }
                    sleep(delay).await;
                    delay *= 2; // Exponential backoff
                }
            }
        }

        unreachable!()
    }
}

/// Helper functions for environment setup and cleanup
pub struct EnvironmentHelper;

impl EnvironmentHelper {
    /// Set environment variables for test duration
    pub fn with_env_vars<F>(env_vars: &[(&str, &str)], test_fn: F)
    where
        F: FnOnce(),
    {
        // Store original values
        let original_values: Vec<(String, Option<String>)> = env_vars
            .iter()
            .map(|(key, _)| (key.to_string(), std::env::var(key).ok()))
            .collect();

        // Set new values
        for (key, value) in env_vars {
            std::env::set_var(key, value);
        }

        // Run test
        test_fn();

        // Restore original values
        for (key, original_value) in original_values {
            match original_value {
                Some(value) => std::env::set_var(key, value),
                None => std::env::remove_var(key),
            }
        }
    }

    /// Get current working directory and restore it after test
    pub fn with_working_directory<F>(new_dir: &Path, test_fn: F) -> std::io::Result<()>
    where
        F: FnOnce() -> std::io::Result<()>,
    {
        let original_dir = std::env::current_dir()?;

        std::env::set_current_dir(new_dir)?;
        let result = test_fn();
        std::env::set_current_dir(original_dir)?;

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::test_fixtures::{ManifestFixture, WorkspaceFixture};
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_filesystem_helper_wait_for_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        // File doesn't exist yet
        let exists = FileSystemHelper::wait_for_file(&file_path, Duration::from_millis(50)).await;
        assert!(!exists);

        // Create file and check again
        std::fs::write(&file_path, "test content").unwrap();
        let exists = FileSystemHelper::wait_for_file(&file_path, Duration::from_millis(50)).await;
        assert!(exists);
    }

    #[tokio::test]
    async fn test_filesystem_helper_file_operations() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = FileSystemHelper::create_temp_file(&temp_dir, "test.txt", "hello world");

        assert!(file_path.exists());

        let content = FileSystemHelper::read_file_content(&file_path).unwrap();
        assert_eq!(content, "hello world");

        let size = FileSystemHelper::get_file_size(&file_path).unwrap();
        assert_eq!(size, 11); // "hello world" is 11 bytes
    }

    #[test]
    fn test_workspace_helper_setup() {
        let temp_dir = TempDir::new().unwrap();
        let workspace = WorkspaceFixture::basic(&temp_dir);

        WorkspaceHelper::setup_test_workspace(&temp_dir, &workspace).unwrap();

        assert!(WorkspaceHelper::verify_workspace_structure(&workspace));
        assert!(workspace.tsrc_dir().exists());
        assert!(workspace.config_path().exists());
        assert!(workspace.manifest_file_path().exists());
    }

    #[test]
    fn test_git_url_helper_operations() {
        let url = GitUrl::new("https://github.com/example/repo.git").unwrap();
        let (https_url, ssh_url) = GitUrlHelper::convert_url_formats(&url);

        assert_eq!(https_url, "https://github.com/example/repo.git");
        assert_eq!(ssh_url, "git@github.com:example/repo.git");

        let (org, repo) = GitUrlHelper::extract_repo_info(&url);
        assert_eq!(org, Some("example".to_string()));
        assert_eq!(repo, Some("repo".to_string()));
    }

    #[test]
    fn test_git_url_helper_test_urls() {
        let urls = GitUrlHelper::create_test_urls("testorg", "testrepo");
        assert_eq!(urls.len(), 4);

        // All URLs should point to the same repository
        for i in 1..urls.len() {
            assert!(GitUrlHelper::urls_are_same_repo(&urls[0], &urls[i]));
        }
    }

    #[tokio::test]
    async fn test_async_helper_timeout() {
        let result = AsyncHelper::with_timeout(
            async {
                sleep(Duration::from_millis(10)).await;
                42
            },
            Duration::from_millis(50),
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);

        let result = AsyncHelper::with_timeout(
            async {
                sleep(Duration::from_millis(100)).await;
                42
            },
            Duration::from_millis(10),
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_async_helper_wait_for_condition() {
        let mut counter = 0;
        let condition = || {
            counter += 1;
            counter >= 3
        };

        let result = AsyncHelper::wait_for_condition(
            condition,
            Duration::from_millis(100),
            Duration::from_millis(10),
        )
        .await;

        assert!(result);
    }

    #[test]
    fn test_environment_helper_env_vars() {
        let test_vars = [("TEST_VAR", "test_value")];

        EnvironmentHelper::with_env_vars(&test_vars, || {
            assert_eq!(std::env::var("TEST_VAR").unwrap(), "test_value");
        });

        // Variable should be cleaned up
        assert!(std::env::var("TEST_VAR").is_err());
    }

    #[test]
    fn test_environment_helper_working_directory() {
        let temp_dir = TempDir::new().unwrap();
        let original_dir = std::env::current_dir().unwrap();

        EnvironmentHelper::with_working_directory(temp_dir.path(), || {
            let current_dir = std::env::current_dir().unwrap();
            assert_eq!(current_dir, temp_dir.path());
            Ok(())
        })
        .unwrap();

        // Should be restored
        let current_dir = std::env::current_dir().unwrap();
        assert_eq!(current_dir, original_dir);
    }
}
