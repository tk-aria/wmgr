//! Integration tests for test helpers
//! 
//! This module tests the test helper functionality to ensure
//! our testing utilities work correctly.

mod common;

use common::{
    test_fixtures::{ManifestFixture, WorkspaceFixture, RepositoryFixture, FileSystemFixture, ValueObjectFixture},
    test_helpers::{FileSystemHelper, WorkspaceHelper, GitUrlHelper, AsyncHelper, EnvironmentHelper},
    assertion_helpers::AssertionHelpers,
    mock_services::{MockManifestService, MockGitRepository, MockCommandExecutor, MockFileSystem},
};
use tempfile::TempDir;
use std::time::Duration;

#[test]
fn test_manifest_fixtures() {
    // Test simple manifest
    let manifest = ManifestFixture::simple();
    assert_eq!(manifest.repos.len(), 2);
    assert_eq!(manifest.repos[0].dest, "repo1");
    assert_eq!(manifest.repos[1].dest, "repo2");
    
    // Test complex manifest
    let manifest = ManifestFixture::complex();
    assert_eq!(manifest.repos.len(), 4);
    assert!(manifest.groups.is_some());
    
    let groups = manifest.groups.unwrap();
    assert_eq!(groups.len(), 5);
    assert!(groups.contains_key("backend"));
    assert!(groups.contains_key("frontend"));
    assert!(groups.contains_key("core"));
}

#[test]
fn test_workspace_fixtures() {
    let temp_dir = TempDir::new().unwrap();
    
    // Test basic workspace
    let workspace = WorkspaceFixture::basic(&temp_dir);
    assert_eq!(workspace.root_path, temp_dir.path());
    assert!(workspace.is_initialized());
    assert!(workspace.manifest.is_some());
    
    // Test uninitialized workspace
    let uninitialized = WorkspaceFixture::uninitialized(&temp_dir);
    assert!(!uninitialized.is_initialized());
}

#[test]
fn test_repository_fixtures() {
    // Test basic repository
    let repo = RepositoryFixture::basic("test-repo");
    assert_eq!(repo.dest, "test-repo");
    assert_eq!(repo.remotes.len(), 1);
    assert_eq!(repo.remotes[0].name, "origin");
    
    // Test repository with multiple remotes
    let repo = RepositoryFixture::with_multiple_remotes("test-repo");
    assert_eq!(repo.dest, "test-repo");
    assert_eq!(repo.remotes.len(), 3);
    assert!(repo.get_remote("origin").is_some());
    assert!(repo.get_remote("upstream").is_some());
    assert!(repo.get_remote("fork").is_some());
}

#[test]
fn test_filesystem_fixtures() {
    let temp_dir = TempDir::new().unwrap();
    
    // Test workspace structure creation
    FileSystemFixture::create_workspace_structure(temp_dir.path()).unwrap();
    assert!(temp_dir.path().join(".tsrc").exists());
    assert!(temp_dir.path().join(".tsrc").join("config.yml").exists());
    assert!(temp_dir.path().join(".tsrc").join("manifest.yml").exists());
    
    // Test repository structure creation
    let repo_path = temp_dir.path().join("test-repo");
    FileSystemFixture::create_repository_structure(&repo_path, "test-repo").unwrap();
    assert!(repo_path.join("README.md").exists());
    assert!(repo_path.join("src").join("main.rs").exists());
    
    // Test Git structure creation
    let git_repo_path = temp_dir.path().join("git-repo");
    std::fs::create_dir_all(&git_repo_path).unwrap();
    FileSystemFixture::create_git_structure(&git_repo_path).unwrap();
    assert!(git_repo_path.join(".git").exists());
    assert!(git_repo_path.join(".git").join("HEAD").exists());
}

#[test]
fn test_value_object_fixtures() {
    // Test Git URLs
    let git_urls = ValueObjectFixture::git_urls();
    assert_eq!(git_urls.len(), 4);
    
    for (url, description) in git_urls {
        assert!(!description.is_empty());
        assert!(url.as_str().contains("example/repo"));
    }
    
    // Test branch names
    let branch_names = ValueObjectFixture::branch_names();
    assert_eq!(branch_names.len(), 5);
    
    for (branch, description) in branch_names {
        assert!(!description.is_empty());
        assert!(!branch.as_str().is_empty());
    }
}

#[tokio::test]
async fn test_filesystem_helpers() {
    let temp_dir = TempDir::new().unwrap();
    
    // Test file operations
    let file_path = FileSystemHelper::create_temp_file(&temp_dir, "test.txt", "hello world");
    assert!(file_path.exists());
    
    let content = FileSystemHelper::read_file_content(&file_path).unwrap();
    assert_eq!(content, "hello world");
    
    let size = FileSystemHelper::get_file_size(&file_path).unwrap();
    assert_eq!(size, 11);
    
    // Test waiting for file
    let new_file_path = temp_dir.path().join("new_file.txt");
    let exists = FileSystemHelper::wait_for_file(&new_file_path, Duration::from_millis(50)).await;
    assert!(!exists); // File doesn't exist
    
    // Create file and check again
    std::fs::write(&new_file_path, "new content").unwrap();
    let exists = FileSystemHelper::wait_for_file(&new_file_path, Duration::from_millis(50)).await;
    assert!(exists);
}

#[test]
fn test_workspace_helpers() {
    let temp_dir = TempDir::new().unwrap();
    let workspace = WorkspaceFixture::basic(&temp_dir);
    
    // Test workspace setup
    WorkspaceHelper::setup_test_workspace(&temp_dir, &workspace).unwrap();
    
    // Verify structure
    assert!(WorkspaceHelper::verify_workspace_structure(&workspace));
    assert!(workspace.tsrc_dir().exists());
    assert!(workspace.config_path().exists());
    assert!(workspace.manifest_file_path().exists());
    
    // Test getting existing repository paths
    let repo_paths = WorkspaceHelper::get_existing_repository_paths(&workspace);
    assert_eq!(repo_paths.len(), 2); // Should have 2 repos from basic fixture
}

#[test]
fn test_git_url_helpers() {
    let urls = GitUrlHelper::create_test_urls("testorg", "testrepo");
    assert_eq!(urls.len(), 4);
    
    // Test URL conversion
    let (https_url, ssh_url) = GitUrlHelper::convert_url_formats(&urls[0]);
    assert!(https_url.contains("https://"));
    assert!(ssh_url.contains("git@"));
    
    // Test same repo check
    for i in 1..urls.len() {
        assert!(GitUrlHelper::urls_are_same_repo(&urls[0], &urls[i]));
    }
    
    // Test repo info extraction
    let (org, repo) = GitUrlHelper::extract_repo_info(&urls[0]);
    assert_eq!(org, Some("testorg".to_string()));
    assert_eq!(repo, Some("testrepo".to_string()));
}

#[tokio::test]
async fn test_async_helpers() {
    // Test timeout
    let result = AsyncHelper::with_timeout(
        async { 42 },
        Duration::from_millis(100)
    ).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
    
    // Test wait for condition
    let mut counter = 0;
    let condition = || {
        counter += 1;
        counter >= 3
    };
    
    let result = AsyncHelper::wait_for_condition(
        condition,
        Duration::from_millis(100),
        Duration::from_millis(10)
    ).await;
    assert!(result);
}

#[test]
fn test_environment_helpers() {
    // Test environment variables
    let test_vars = [("TEST_VAR", "test_value")];
    
    EnvironmentHelper::with_env_vars(&test_vars, || {
        assert_eq!(std::env::var("TEST_VAR").unwrap(), "test_value");
    });
    
    // Variable should be cleaned up
    assert!(std::env::var("TEST_VAR").is_err());
    
    // Test working directory
    let temp_dir = TempDir::new().unwrap();
    let original_dir = std::env::current_dir().unwrap();
    
    EnvironmentHelper::with_working_directory(temp_dir.path(), || {
        let current_dir = std::env::current_dir().unwrap();
        assert_eq!(current_dir, temp_dir.path());
        Ok(())
    }).unwrap();
    
    // Should be restored
    let current_dir = std::env::current_dir().unwrap();
    assert_eq!(current_dir, original_dir);
}

#[tokio::test]
async fn test_mock_services() {
    // Test mock manifest service
    let mock_service = MockManifestService::new();
    let manifest = ManifestFixture::simple();
    
    mock_service.add_manifest("/test/manifest.yml", manifest.clone());
    
    let result = mock_service.parse_from_file(std::path::Path::new("/test/manifest.yml")).await;
    assert!(result.is_ok());
    
    let history = mock_service.get_call_history();
    assert_eq!(history.len(), 1);
    assert!(history[0].contains("parse_from_file"));
    
    // Test mock command executor
    let mock_executor = MockCommandExecutor::new();
    mock_executor.set_command_result("echo hello", 0, "hello\n", "");
    
    let result = mock_executor.execute("echo hello").await;
    assert!(result.is_ok());
    
    let (exit_code, stdout, stderr) = result.unwrap();
    assert_eq!(exit_code, 0);
    assert_eq!(stdout, "hello\n");
    assert_eq!(stderr, "");
    
    // Test mock file system
    let mock_fs = MockFileSystem::new();
    let file_path = std::path::Path::new("/test/file.txt");
    
    mock_fs.add_file(file_path, "test content");
    assert!(mock_fs.file_exists(file_path));
    
    let content = mock_fs.read_file(file_path).unwrap();
    assert_eq!(content, "test content");
}

#[test]
fn test_assertion_helpers() {
    let temp_dir = TempDir::new().unwrap();
    
    // Test file assertions using macros
    let file_path = temp_dir.path().join("test.txt");
    std::fs::write(&file_path, "test content").unwrap();
    
    assert_file_exists!(&file_path);
    assert_file_content!(&file_path, "test content");
    assert_file_contains!(&file_path, "test");
    
    // Test directory assertions
    let dir_path = temp_dir.path().join("subdir");
    std::fs::create_dir_all(&dir_path).unwrap();
    assert_dir_exists!(&dir_path);
    
    // Test workspace structure assertions
    let workspace = WorkspaceFixture::basic(&temp_dir);
    WorkspaceHelper::setup_test_workspace(&temp_dir, &workspace).unwrap();
    
    let expected_repos = vec!["repo1", "repo2"];
    AssertionHelpers::assert_workspace_structure(&workspace, &expected_repos);
    
    // Test manifest assertions
    let manifest = ManifestFixture::complex();
    let expected_repos = vec![
        ("backend/api", "https://github.com/example/backend-api.git"),
        ("frontend/web", "https://github.com/example/frontend-web.git"),
        ("tools/scripts", "https://github.com/example/build-tools.git"),
        ("docs/wiki", "https://github.com/example/documentation.git"),
    ];
    AssertionHelpers::assert_manifest_repos(&manifest, &expected_repos);
    
    let backend_repos = vec!["backend/api"];
    let frontend_repos = vec!["frontend/web"];
    let core_repos = vec!["backend/api", "frontend/web"];
    let expected_groups = vec![
        ("backend", backend_repos.as_slice()),
        ("frontend", frontend_repos.as_slice()),
        ("core", core_repos.as_slice()),
    ];
    AssertionHelpers::assert_manifest_groups(&manifest, &expected_groups);
}