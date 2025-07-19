//! Test fixtures for creating test data
//!
//! This module provides reusable fixtures for creating test entities,
//! workspace configurations, and file structures.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use tsrc::domain::{
    entities::{
        manifest::{Group, Manifest, ManifestRepo},
        repository::{Remote, Repository},
        workspace::{Workspace, WorkspaceConfig, WorkspaceStatus},
    },
    value_objects::{branch_name::BranchName, file_path::FilePath, git_url::GitUrl},
};

/// Test fixture for creating common manifest configurations
pub struct ManifestFixture;

impl ManifestFixture {
    /// Create a simple manifest with basic repositories
    pub fn simple() -> Manifest {
        let repos = vec![
            ManifestRepo::new("https://github.com/example/repo1.git", "repo1"),
            ManifestRepo::new("https://github.com/example/repo2.git", "repo2"),
        ];

        Manifest::new(repos)
    }

    /// Create a complex manifest with groups and multiple repositories
    pub fn complex() -> Manifest {
        let repos = vec![
            ManifestRepo::new("https://github.com/example/backend-api.git", "backend/api")
                .with_branch("main"),
            ManifestRepo::new(
                "https://github.com/example/frontend-web.git",
                "frontend/web",
            )
            .with_branch("develop"),
            ManifestRepo::new(
                "https://github.com/example/build-tools.git",
                "tools/scripts",
            )
            .with_branch("main"),
            ManifestRepo::new("https://github.com/example/documentation.git", "docs/wiki")
                .with_branch("main"),
        ];

        let mut groups = HashMap::new();
        groups.insert(
            "backend".to_string(),
            Group::new(vec!["backend/api".to_string()]).with_description("Backend services"),
        );
        groups.insert(
            "frontend".to_string(),
            Group::new(vec!["frontend/web".to_string()]).with_description("Frontend applications"),
        );
        groups.insert(
            "core".to_string(),
            Group::new(vec!["backend/api".to_string(), "frontend/web".to_string()])
                .with_description("Core application components"),
        );
        groups.insert(
            "tools".to_string(),
            Group::new(vec!["tools/scripts".to_string()]).with_description("Development tools"),
        );
        groups.insert(
            "docs".to_string(),
            Group::new(vec!["docs/wiki".to_string()]).with_description("Documentation"),
        );

        Manifest::new(repos)
            .with_groups(groups)
            .with_default_branch("main")
    }

    /// Create a manifest with specific groups
    pub fn with_groups(group_config: Vec<(&str, Vec<&str>, Option<&str>)>) -> Manifest {
        let mut repos = Vec::new();
        let mut groups = HashMap::new();

        for (group_name, repo_names, description) in group_config {
            let mut group_repos = Vec::new();

            for repo_name in repo_names {
                let repo_url = format!("https://github.com/example/{}.git", repo_name);
                repos.push(ManifestRepo::new(&repo_url, repo_name));
                group_repos.push(repo_name.to_string());
            }

            let mut group = Group::new(group_repos);
            if let Some(desc) = description {
                group = group.with_description(desc);
            }
            groups.insert(group_name.to_string(), group);
        }

        Manifest::new(repos).with_groups(groups)
    }
}

/// Test fixture for creating workspace configurations
pub struct WorkspaceFixture;

impl WorkspaceFixture {
    /// Create a basic workspace in a temporary directory
    pub fn basic(temp_dir: &TempDir) -> Workspace {
        let workspace_path = temp_dir.path().to_path_buf();
        let config = WorkspaceConfig::new("https://github.com/example/manifest.git", "main");

        let manifest = ManifestFixture::simple();

        Workspace::new(workspace_path, config)
            .with_manifest(manifest)
            .with_status(WorkspaceStatus::Initialized)
    }

    /// Create a workspace with complex manifest configuration
    pub fn complex(temp_dir: &TempDir) -> Workspace {
        let workspace_path = temp_dir.path().to_path_buf();
        let config = WorkspaceConfig::new("https://github.com/example/manifest.git", "main");

        let manifest = ManifestFixture::complex();

        Workspace::new(workspace_path, config)
            .with_manifest(manifest)
            .with_status(WorkspaceStatus::Initialized)
    }

    /// Create an uninitialized workspace
    pub fn uninitialized(temp_dir: &TempDir) -> Workspace {
        let workspace_path = temp_dir.path().to_path_buf();
        let config = WorkspaceConfig::new("https://github.com/example/manifest.git", "main");

        Workspace::new(workspace_path, config)
    }
}

/// Test fixture for creating repository structures
pub struct RepositoryFixture;

impl RepositoryFixture {
    /// Create a basic repository with origin remote
    pub fn basic(dest: &str) -> Repository {
        let remotes = vec![Remote::new(
            "origin",
            &format!("https://github.com/example/{}.git", dest),
        )];

        Repository::new(dest, remotes)
    }

    /// Create a repository with multiple remotes
    pub fn with_multiple_remotes(dest: &str) -> Repository {
        let remotes = vec![
            Remote::new(
                "origin",
                &format!("https://github.com/example/{}.git", dest),
            ),
            Remote::new(
                "upstream",
                &format!("https://github.com/upstream/{}.git", dest),
            ),
            Remote::new("fork", &format!("https://github.com/user/{}.git", dest)),
        ];

        Repository::new(dest, remotes)
    }
}

/// Test fixture for creating file system structures
pub struct FileSystemFixture;

impl FileSystemFixture {
    /// Create a basic workspace directory structure
    pub fn create_workspace_structure(workspace_path: &Path) -> std::io::Result<()> {
        let tsrc_dir = workspace_path.join(".tsrc");
        std::fs::create_dir_all(&tsrc_dir)?;

        // Create basic config and manifest files
        let config_content = r#"manifest_url: https://github.com/example/manifest.git
manifest_branch: main
groups: []
shallow: false
"#;
        std::fs::write(tsrc_dir.join("config.yml"), config_content)?;

        let manifest_content = r#"repos:
  - dest: example-repo
    url: https://github.com/example/repo.git
    branch: main
"#;
        std::fs::write(tsrc_dir.join("manifest.yml"), manifest_content)?;

        Ok(())
    }

    /// Create a repository directory with basic files
    pub fn create_repository_structure(repo_path: &Path, repo_name: &str) -> std::io::Result<()> {
        std::fs::create_dir_all(repo_path)?;

        // Create basic files
        std::fs::write(repo_path.join("README.md"), format!("# {}\n", repo_name))?;
        std::fs::write(repo_path.join(".gitignore"), "target/\n*.log\n")?;

        // Create source directory
        let src_dir = repo_path.join("src");
        std::fs::create_dir_all(&src_dir)?;
        std::fs::write(
            src_dir.join("main.rs"),
            "fn main() {\n    println!(\"Hello, world!\");\n}\n",
        )?;

        Ok(())
    }

    /// Create a Git repository structure (simplified)
    pub fn create_git_structure(repo_path: &Path) -> std::io::Result<()> {
        let git_dir = repo_path.join(".git");
        std::fs::create_dir_all(&git_dir)?;

        // Create basic Git structure
        std::fs::write(git_dir.join("HEAD"), "ref: refs/heads/main\n")?;

        let refs_dir = git_dir.join("refs").join("heads");
        std::fs::create_dir_all(&refs_dir)?;
        std::fs::write(refs_dir.join("main"), "dummy-commit-hash\n")?;

        let objects_dir = git_dir.join("objects");
        std::fs::create_dir_all(objects_dir.join("info"))?;
        std::fs::create_dir_all(objects_dir.join("pack"))?;

        Ok(())
    }
}

/// Test fixture for creating value objects
pub struct ValueObjectFixture;

impl ValueObjectFixture {
    /// Create common Git URLs for testing
    pub fn git_urls() -> Vec<(GitUrl, &'static str)> {
        vec![
            (
                GitUrl::new("https://github.com/example/repo.git").unwrap(),
                "HTTPS URL",
            ),
            (
                GitUrl::new("git@github.com:example/repo.git").unwrap(),
                "SSH URL",
            ),
            (
                GitUrl::new("https://gitlab.com/example/repo.git").unwrap(),
                "GitLab HTTPS",
            ),
            (
                GitUrl::new("git@gitlab.com:example/repo.git").unwrap(),
                "GitLab SSH",
            ),
        ]
    }

    /// Create common file paths for testing
    pub fn file_paths() -> Vec<(FilePath, &'static str)> {
        vec![
            (
                FilePath::new_relative("src/main.rs").unwrap(),
                "Relative file",
            ),
            (
                FilePath::new_relative("docs/README.md").unwrap(),
                "Relative nested file",
            ),
        ]
    }

    /// Create common branch names for testing
    pub fn branch_names() -> Vec<(BranchName, &'static str)> {
        vec![
            (BranchName::new("main").unwrap(), "Main branch"),
            (BranchName::new("develop").unwrap(), "Develop branch"),
            (
                BranchName::new("feature/user-auth").unwrap(),
                "Feature branch",
            ),
            (
                BranchName::new("hotfix/security-patch").unwrap(),
                "Hotfix branch",
            ),
            (BranchName::new("release/v1.0.0").unwrap(), "Release branch"),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_manifest_fixture_simple() {
        let manifest = ManifestFixture::simple();
        assert_eq!(manifest.repos.len(), 2);
        assert_eq!(manifest.repos[0].dest, "repo1");
        assert_eq!(manifest.repos[1].dest, "repo2");
    }

    #[test]
    fn test_manifest_fixture_complex() {
        let manifest = ManifestFixture::complex();
        assert_eq!(manifest.repos.len(), 4);

        let groups = manifest.groups.unwrap();
        assert_eq!(groups.len(), 5);
        assert!(groups.contains_key("backend"));
        assert!(groups.contains_key("frontend"));
        assert!(groups.contains_key("core"));
        assert!(groups.contains_key("tools"));
        assert!(groups.contains_key("docs"));
    }

    #[test]
    fn test_manifest_fixture_with_groups() {
        let group_config = vec![
            ("web", vec!["frontend", "backend"], Some("Web components")),
            ("tools", vec!["scripts", "utils"], Some("Development tools")),
        ];

        let manifest = ManifestFixture::with_groups(group_config);
        assert_eq!(manifest.repos.len(), 4);

        let groups = manifest.groups.unwrap();
        assert_eq!(groups.len(), 2);
        assert!(groups.contains_key("web"));
        assert!(groups.contains_key("tools"));
    }

    #[test]
    fn test_workspace_fixture_basic() {
        let temp_dir = TempDir::new().unwrap();
        let workspace = WorkspaceFixture::basic(&temp_dir);

        assert_eq!(workspace.root_path, temp_dir.path());
        assert!(workspace.is_initialized());
        assert!(workspace.manifest.is_some());
    }

    #[test]
    fn test_repository_fixture_basic() {
        let repo = RepositoryFixture::basic("test-repo");
        assert_eq!(repo.dest, "test-repo");
        assert_eq!(repo.remotes.len(), 1);
        assert_eq!(repo.remotes[0].name, "origin");
    }

    #[test]
    fn test_repository_fixture_multiple_remotes() {
        let repo = RepositoryFixture::with_multiple_remotes("test-repo");
        assert_eq!(repo.dest, "test-repo");
        assert_eq!(repo.remotes.len(), 3);
        assert!(repo.get_remote("origin").is_some());
        assert!(repo.get_remote("upstream").is_some());
        assert!(repo.get_remote("fork").is_some());
    }

    #[test]
    fn test_filesystem_fixture_workspace_structure() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_path = temp_dir.path();

        FileSystemFixture::create_workspace_structure(workspace_path).unwrap();

        assert!(workspace_path.join(".tsrc").exists());
        assert!(workspace_path.join(".tsrc").join("config.yml").exists());
        assert!(workspace_path.join(".tsrc").join("manifest.yml").exists());
    }

    #[test]
    fn test_filesystem_fixture_repository_structure() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().join("test-repo");

        FileSystemFixture::create_repository_structure(&repo_path, "test-repo").unwrap();

        assert!(repo_path.join("README.md").exists());
        assert!(repo_path.join(".gitignore").exists());
        assert!(repo_path.join("src").join("main.rs").exists());
    }

    #[test]
    fn test_filesystem_fixture_git_structure() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path().join("test-repo");
        std::fs::create_dir_all(&repo_path).unwrap();

        FileSystemFixture::create_git_structure(&repo_path).unwrap();

        assert!(repo_path.join(".git").exists());
        assert!(repo_path.join(".git").join("HEAD").exists());
        assert!(repo_path
            .join(".git")
            .join("refs")
            .join("heads")
            .join("main")
            .exists());
    }

    #[test]
    fn test_value_object_fixture_git_urls() {
        let git_urls = ValueObjectFixture::git_urls();
        assert_eq!(git_urls.len(), 4);

        for (url, description) in git_urls {
            assert!(!description.is_empty());
            assert!(url.as_str().contains("example/repo"));
        }
    }

    #[test]
    fn test_value_object_fixture_branch_names() {
        let branch_names = ValueObjectFixture::branch_names();
        assert_eq!(branch_names.len(), 5);

        for (branch, description) in branch_names {
            assert!(!description.is_empty());
            assert!(!branch.as_str().is_empty());
        }
    }
}
