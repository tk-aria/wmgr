//! Assertion helpers for testing
//!
//! This module provides custom assertion macros and helper functions
//! that make test assertions more readable and provide better error messages.

use std::path::Path;
use std::time::Duration;
use tsrc::domain::{
    entities::{manifest::Manifest, repository::Repository, workspace::Workspace},
    value_objects::git_url::GitUrl,
};

/// Assert that a file exists
#[macro_export]
macro_rules! assert_file_exists {
    ($path:expr) => {
        assert!($path.exists(), "File should exist: {}", $path.display());
    };
    ($path:expr, $msg:expr) => {
        assert!($path.exists(), "{}: {}", $msg, $path.display());
    };
}

/// Assert that a file does not exist
#[macro_export]
macro_rules! assert_file_not_exists {
    ($path:expr) => {
        assert!(
            !$path.exists(),
            "File should not exist: {}",
            $path.display()
        );
    };
    ($path:expr, $msg:expr) => {
        assert!(!$path.exists(), "{}: {}", $msg, $path.display());
    };
}

/// Assert that a directory exists
#[macro_export]
macro_rules! assert_dir_exists {
    ($path:expr) => {
        assert!(
            $path.exists() && $path.is_dir(),
            "Directory should exist: {}",
            $path.display()
        );
    };
    ($path:expr, $msg:expr) => {
        assert!(
            $path.exists() && $path.is_dir(),
            "{}: {}",
            $msg,
            $path.display()
        );
    };
}

/// Assert that file content matches expected content
#[macro_export]
macro_rules! assert_file_content {
    ($path:expr, $expected:expr) => {
        let content = std::fs::read_to_string($path)
            .expect(&format!("Failed to read file: {}", $path.display()));
        assert_eq!(
            content.trim(),
            $expected.trim(),
            "File content mismatch in: {}",
            $path.display()
        );
    };
}

/// Assert that file content contains expected substring
#[macro_export]
macro_rules! assert_file_contains {
    ($path:expr, $expected:expr) => {
        let content = std::fs::read_to_string($path)
            .expect(&format!("Failed to read file: {}", $path.display()));
        assert!(
            content.contains($expected),
            "File should contain '{}' in: {}",
            $expected,
            $path.display()
        );
    };
}

/// Assert that a collection contains a specific item
#[macro_export]
macro_rules! assert_contains {
    ($collection:expr, $item:expr) => {
        assert!(
            $collection.contains($item),
            "Collection should contain item: {:?}",
            $item
        );
    };
    ($collection:expr, $item:expr, $msg:expr) => {
        assert!($collection.contains($item), "{}: {:?}", $msg, $item);
    };
}

/// Assert that a result is Ok and return the value
#[macro_export]
macro_rules! assert_ok {
    ($result:expr) => {
        match $result {
            Ok(value) => value,
            Err(err) => panic!("Expected Ok, got Err: {:?}", err),
        }
    };
    ($result:expr, $msg:expr) => {
        match $result {
            Ok(value) => value,
            Err(err) => panic!("{}: {:?}", $msg, err),
        }
    };
}

/// Assert that a result is Err
#[macro_export]
macro_rules! assert_err {
    ($result:expr) => {
        match $result {
            Ok(value) => panic!("Expected Err, got Ok: {:?}", value),
            Err(_) => (),
        }
    };
    ($result:expr, $msg:expr) => {
        match $result {
            Ok(value) => panic!("{}: {:?}", $msg, value),
            Err(_) => (),
        }
    };
}

/// Assert that execution completes within a timeout
#[macro_export]
macro_rules! assert_timeout {
    ($duration:expr, $block:block) => {
        let start = std::time::Instant::now();
        $block
        let elapsed = start.elapsed();
        assert!(
            elapsed <= $duration,
            "Operation took too long: {:?} > {:?}",
            elapsed,
            $duration
        );
    };
}

/// Custom assertion helpers for domain-specific objects
pub struct AssertionHelpers;

impl AssertionHelpers {
    /// Assert that workspace has the expected structure
    pub fn assert_workspace_structure(workspace: &Workspace, expected_repos: &[&str]) {
        assert!(
            workspace.is_initialized(),
            "Workspace should be initialized"
        );

        if let Some(manifest) = &workspace.manifest {
            assert_eq!(
                manifest.repos.len(),
                expected_repos.len(),
                "Expected {} repositories, found {}",
                expected_repos.len(),
                manifest.repos.len()
            );

            for expected_repo in expected_repos {
                assert!(
                    manifest.repos.iter().any(|r| r.dest == *expected_repo),
                    "Repository '{}' not found in manifest",
                    expected_repo
                );
            }
        } else {
            panic!("Workspace should have a manifest");
        }

        // Check directory structure
        assert_file_exists!(workspace.tsrc_dir(), "tsrc directory should exist");
        assert_file_exists!(workspace.config_path(), "config file should exist");

        if workspace.manifest.is_some() {
            assert_file_exists!(workspace.manifest_file_path(), "manifest file should exist");
        }
    }

    /// Assert that manifest has expected repositories
    pub fn assert_manifest_repos(manifest: &Manifest, expected_repos: &[(&str, &str)]) {
        assert_eq!(
            manifest.repos.len(),
            expected_repos.len(),
            "Expected {} repositories, found {}",
            expected_repos.len(),
            manifest.repos.len()
        );

        for (expected_dest, expected_url) in expected_repos {
            let repo = manifest.repos.iter().find(|r| r.dest == *expected_dest);
            assert!(
                repo.is_some(),
                "Repository with dest '{}' not found",
                expected_dest
            );

            let repo = repo.unwrap();
            assert_eq!(
                repo.url, *expected_url,
                "Repository '{}' has wrong URL. Expected: {}, Found: {}",
                expected_dest, expected_url, repo.url
            );
        }
    }

    /// Assert that manifest has expected groups
    pub fn assert_manifest_groups(manifest: &Manifest, expected_groups: &[(&str, &[&str])]) {
        let groups = manifest
            .groups
            .as_ref()
            .expect("Manifest should have groups");

        assert_eq!(
            groups.len(),
            expected_groups.len(),
            "Expected {} groups, found {}",
            expected_groups.len(),
            groups.len()
        );

        for (expected_name, expected_repos) in expected_groups {
            let group = groups.get(*expected_name);
            assert!(group.is_some(), "Group '{}' not found", expected_name);

            let group = group.unwrap();
            assert_eq!(
                group.repos.len(),
                expected_repos.len(),
                "Group '{}' should have {} repositories, found {}",
                expected_name,
                expected_repos.len(),
                group.repos.len()
            );

            for expected_repo in *expected_repos {
                assert!(
                    group.repos.contains(&expected_repo.to_string()),
                    "Group '{}' should contain repository '{}'",
                    expected_name,
                    expected_repo
                );
            }
        }
    }

    /// Assert that repository has expected remotes
    pub fn assert_repository_remotes(repository: &Repository, expected_remotes: &[(&str, &str)]) {
        assert_eq!(
            repository.remotes.len(),
            expected_remotes.len(),
            "Expected {} remotes, found {}",
            expected_remotes.len(),
            repository.remotes.len()
        );

        for (expected_name, expected_url) in expected_remotes {
            let remote = repository.get_remote(expected_name);
            assert!(remote.is_some(), "Remote '{}' not found", expected_name);

            let remote = remote.unwrap();
            assert_eq!(
                remote.url, *expected_url,
                "Remote '{}' has wrong URL. Expected: {}, Found: {}",
                expected_name, expected_url, remote.url
            );
        }
    }

    /// Assert that Git URLs represent the same repository
    pub fn assert_same_repository(url1: &GitUrl, url2: &GitUrl) {
        assert!(
            url1.is_same_repo(url2),
            "URLs should represent the same repository: {} vs {}",
            url1.as_str(),
            url2.as_str()
        );
    }

    /// Assert that Git URL has expected components
    pub fn assert_git_url_components(
        url: &GitUrl,
        expected_org: Option<&str>,
        expected_repo: Option<&str>,
    ) {
        assert_eq!(
            url.organization().as_deref(),
            expected_org,
            "URL organization mismatch for: {}",
            url.as_str()
        );

        assert_eq!(
            url.repo_name().as_deref(),
            expected_repo,
            "URL repository name mismatch for: {}",
            url.as_str()
        );
    }

    /// Assert that directory contains expected files
    pub fn assert_directory_contents(dir: &Path, expected_files: &[&str]) {
        assert_dir_exists!(dir, "Directory should exist");

        let entries: Result<Vec<_>, _> = std::fs::read_dir(dir)
            .expect("Failed to read directory")
            .collect();

        let entries = entries.expect("Failed to collect directory entries");
        let actual_files: Vec<String> = entries
            .iter()
            .filter(|e| e.file_type().unwrap().is_file())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect();

        for expected_file in expected_files {
            assert!(
                actual_files.contains(&expected_file.to_string()),
                "Directory should contain file '{}'. Found files: {:?}",
                expected_file,
                actual_files
            );
        }
    }

    /// Assert that two file contents are equal
    pub fn assert_files_equal(path1: &Path, path2: &Path) {
        assert_file_exists!(path1, "First file should exist");
        assert_file_exists!(path2, "Second file should exist");

        let content1 = std::fs::read_to_string(path1)
            .expect(&format!("Failed to read first file: {}", path1.display()));
        let content2 = std::fs::read_to_string(path2)
            .expect(&format!("Failed to read second file: {}", path2.display()));

        assert_eq!(
            content1.trim(),
            content2.trim(),
            "File contents should be equal:\n{}\nvs\n{}",
            path1.display(),
            path2.display()
        );
    }

    /// Assert that string matches a regex pattern
    pub fn assert_matches_pattern(text: &str, pattern: &str) {
        // Simple pattern matching without regex dependency
        // In a real implementation, you'd use the regex crate
        assert!(
            text.contains(pattern),
            "Text should contain pattern '{}': {}",
            pattern,
            text
        );
    }

    /// Assert that operation completes within timeout
    pub async fn assert_async_timeout<F, T>(future: F, timeout: Duration, operation_name: &str) -> T
    where
        F: std::future::Future<Output = T>,
    {
        match tokio::time::timeout(timeout, future).await {
            Ok(result) => result,
            Err(_) => panic!(
                "Operation '{}' timed out after {:?}",
                operation_name, timeout
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::test_fixtures::{ManifestFixture, RepositoryFixture, WorkspaceFixture};
    use std::collections::HashMap;
    use tempfile::TempDir;

    #[test]
    fn test_file_assertions() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        // File doesn't exist initially
        assert_file_not_exists!(file_path);

        // Create file
        std::fs::write(&file_path, "hello world").unwrap();
        assert_file_exists!(file_path);

        // Check content
        assert_file_content!(file_path, "hello world");
        assert_file_contains!(file_path, "hello");
        assert_file_contains!(file_path, "world");
    }

    #[test]
    fn test_directory_assertions() {
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path().join("subdir");

        std::fs::create_dir_all(&dir_path).unwrap();
        assert_dir_exists!(dir_path);
    }

    #[test]
    fn test_result_assertions() {
        let ok_result: Result<i32, &str> = Ok(42);
        let err_result: Result<i32, &str> = Err("error");

        let value = assert_ok!(ok_result);
        assert_eq!(value, 42);

        assert_err!(err_result);
    }

    #[test]
    fn test_workspace_structure_assertion() {
        let temp_dir = TempDir::new().unwrap();
        let workspace = WorkspaceFixture::basic(&temp_dir);

        // Create the required structure
        std::fs::create_dir_all(workspace.tsrc_dir()).unwrap();
        std::fs::write(workspace.config_path(), "test config").unwrap();
        std::fs::write(workspace.manifest_file_path(), "test manifest").unwrap();

        let expected_repos = vec!["repo1", "repo2"];
        AssertionHelpers::assert_workspace_structure(&workspace, &expected_repos);
    }

    #[test]
    fn test_manifest_repos_assertion() {
        let manifest = ManifestFixture::simple();
        let expected_repos = vec![
            ("repo1", "https://github.com/example/repo1.git"),
            ("repo2", "https://github.com/example/repo2.git"),
        ];

        AssertionHelpers::assert_manifest_repos(&manifest, &expected_repos);
    }

    #[test]
    fn test_manifest_groups_assertion() {
        let manifest = ManifestFixture::complex();
        let expected_groups = vec![
            ("backend", vec!["backend/api"].as_slice()),
            ("frontend", vec!["frontend/web"].as_slice()),
            ("core", vec!["backend/api", "frontend/web"].as_slice()),
        ];

        AssertionHelpers::assert_manifest_groups(&manifest, &expected_groups);
    }

    #[test]
    fn test_repository_remotes_assertion() {
        let repository = RepositoryFixture::with_multiple_remotes("test-repo");
        let expected_remotes = vec![
            ("origin", "https://github.com/example/test-repo.git"),
            ("upstream", "https://github.com/upstream/test-repo.git"),
            ("fork", "https://github.com/user/test-repo.git"),
        ];

        AssertionHelpers::assert_repository_remotes(&repository, &expected_remotes);
    }

    #[test]
    fn test_git_url_assertions() {
        let url1 = GitUrl::new("https://github.com/example/repo.git").unwrap();
        let url2 = GitUrl::new("git@github.com:example/repo.git").unwrap();

        AssertionHelpers::assert_same_repository(&url1, &url2);
        AssertionHelpers::assert_git_url_components(&url1, Some("example"), Some("repo"));
    }

    #[test]
    fn test_directory_contents_assertion() {
        let temp_dir = TempDir::new().unwrap();

        // Create some files
        std::fs::write(temp_dir.path().join("file1.txt"), "content1").unwrap();
        std::fs::write(temp_dir.path().join("file2.txt"), "content2").unwrap();

        let expected_files = vec!["file1.txt", "file2.txt"];
        AssertionHelpers::assert_directory_contents(temp_dir.path(), &expected_files);
    }

    #[test]
    fn test_files_equal_assertion() {
        let temp_dir = TempDir::new().unwrap();
        let file1 = temp_dir.path().join("file1.txt");
        let file2 = temp_dir.path().join("file2.txt");

        std::fs::write(&file1, "same content").unwrap();
        std::fs::write(&file2, "same content").unwrap();

        AssertionHelpers::assert_files_equal(&file1, &file2);
    }

    #[tokio::test]
    async fn test_async_timeout_assertion() {
        let result = AssertionHelpers::assert_async_timeout(
            async { 42 },
            Duration::from_millis(100),
            "test operation",
        )
        .await;

        assert_eq!(result, 42);
    }
}
