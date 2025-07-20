use super::scm_interface::{AsAny, CloneOptions, ScmError, ScmOperations, StatusResult, SyncOptions};
use crate::domain::value_objects::scm_type::ScmType;
use async_trait::async_trait;
use std::any::Any;
use std::collections::HashMap;
use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;

/// Git implementation of SCM operations
pub struct GitScm {
    git_executable: String,
}

impl Default for GitScm {
    fn default() -> Self {
        Self {
            git_executable: "git".to_string(),
        }
    }
}

impl GitScm {
    /// Create a new Git SCM instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new Git SCM instance with custom executable path
    pub fn with_executable(executable: impl Into<String>) -> Self {
        Self {
            git_executable: executable.into(),
        }
    }

    /// Check if git executable is available
    pub async fn check_availability(&self) -> Result<(), ScmError> {
        let output = Command::new(&self.git_executable)
            .arg("--version")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        if !output.status.success() {
            return Err(ScmError::executable_not_found(&self.git_executable));
        }

        Ok(())
    }

    /// Execute a git command in the given directory
    async fn execute_git_command(
        &self,
        args: &[&str],
        working_dir: Option<&Path>,
    ) -> Result<std::process::Output, ScmError> {
        let mut cmd = Command::new(&self.git_executable);
        cmd.args(args).stdout(Stdio::piped()).stderr(Stdio::piped());

        if let Some(dir) = working_dir {
            cmd.current_dir(dir);
        }

        let output = cmd.output().await?;
        Ok(output)
    }

    /// Execute a git command and check for success
    async fn execute_git_command_checked(
        &self,
        args: &[&str],
        working_dir: Option<&Path>,
    ) -> Result<String, ScmError> {
        let output = self.execute_git_command(args, working_dir).await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let command = format!("{} {}", self.git_executable, args.join(" "));
            return Err(ScmError::command_failed(
                command,
                output.status.code().unwrap_or(-1),
                stderr,
            ));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
}

#[async_trait]
impl ScmOperations for GitScm {
    async fn clone_repository(
        &self,
        url: &str,
        dest_path: &Path,
        options: &CloneOptions,
    ) -> Result<(), ScmError> {
        let mut args = vec!["clone"];

        // Add shallow clone option
        if options.shallow {
            args.push("--depth");
            args.push("1");
        }

        // Add branch option
        if let Some(branch) = &options.branch {
            args.push("--branch");
            args.push(branch);
        }

        // Add URL and destination
        args.push(url);
        args.push(
            dest_path
                .to_str()
                .ok_or_else(|| ScmError::Internal {
                    message: "Invalid destination path".to_string(),
                })?,
        );

        // Add extra options
        for option in &options.extra_options {
            args.push(option);
        }

        let output = self.execute_git_command(&args, None).await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ScmError::clone_failed(format!(
                "Git clone failed: {}",
                stderr
            )));
        }

        // If a specific revision is requested, checkout that revision
        if let Some(revision) = &options.revision {
            self.execute_git_command_checked(&["checkout", revision], Some(dest_path))
                .await
                .map_err(|e| ScmError::clone_failed(format!("Failed to checkout revision: {}", e)))?;
        }

        Ok(())
    }

    async fn sync_repository(
        &self,
        repo_path: &Path,
        options: &SyncOptions,
    ) -> Result<(), ScmError> {
        // Fetch latest changes
        self.execute_git_command_checked(&["fetch", "origin"], Some(repo_path))
            .await
            .map_err(|e| ScmError::sync_failed(format!("Fetch failed: {}", e)))?;

        // Handle force option
        if options.force {
            // Reset to clean state
            self.execute_git_command_checked(&["reset", "--hard"], Some(repo_path))
                .await
                .map_err(|e| ScmError::sync_failed(format!("Reset failed: {}", e)))?;

            // Clean untracked files
            self.execute_git_command_checked(&["clean", "-fd"], Some(repo_path))
                .await
                .map_err(|e| ScmError::sync_failed(format!("Clean failed: {}", e)))?;
        }

        // Checkout target branch or revision
        if let Some(revision) = &options.revision {
            self.execute_git_command_checked(&["checkout", revision], Some(repo_path))
                .await
                .map_err(|e| ScmError::sync_failed(format!("Checkout failed: {}", e)))?;
        } else if let Some(branch) = &options.branch {
            // Switch to target branch
            self.execute_git_command_checked(&["checkout", branch], Some(repo_path))
                .await
                .map_err(|e| ScmError::sync_failed(format!("Branch checkout failed: {}", e)))?;

            // Pull latest changes
            self.execute_git_command_checked(&["pull", "--ff-only"], Some(repo_path))
                .await
                .map_err(|e| ScmError::sync_failed(format!("Pull failed: {}", e)))?;
        } else {
            // Just pull on current branch
            self.execute_git_command_checked(&["pull", "--ff-only"], Some(repo_path))
                .await
                .map_err(|e| ScmError::sync_failed(format!("Pull failed: {}", e)))?;
        }

        Ok(())
    }

    async fn get_status(&self, repo_path: &Path) -> Result<StatusResult, ScmError> {
        // Get current revision
        let current_revision = self
            .execute_git_command_checked(&["rev-parse", "HEAD"], Some(repo_path))
            .await
            .map_err(|e| ScmError::status_failed(format!("Failed to get revision: {}", e)))?;

        // Get current branch
        let current_branch = self
            .execute_git_command_checked(&["branch", "--show-current"], Some(repo_path))
            .await
            .ok()
            .filter(|b| !b.is_empty());

        // Check for uncommitted changes
        let status_output = self
            .execute_git_command_checked(&["status", "--porcelain"], Some(repo_path))
            .await
            .map_err(|e| ScmError::status_failed(format!("Failed to get status: {}", e)))?;

        let has_changes = !status_output.is_empty();

        // Check for untracked files
        let untracked_output = self
            .execute_git_command_checked(&["ls-files", "--others", "--exclude-standard"], Some(repo_path))
            .await
            .map_err(|e| ScmError::status_failed(format!("Failed to check untracked files: {}", e)))?;

        let has_untracked = !untracked_output.is_empty();

        // Get ahead/behind count
        let (ahead_count, behind_count) = if let Some(branch) = &current_branch {
            let upstream_ref = format!("origin/{}", branch);
            
            // Check if upstream exists
            let upstream_exists = self
                .execute_git_command(&["rev-parse", "--verify", &upstream_ref], Some(repo_path))
                .await
                .map(|output| output.status.success())
                .unwrap_or(false);

            if upstream_exists {
                let ahead_output = self
                    .execute_git_command_checked(
                        &["rev-list", "--count", &format!("{}..HEAD", upstream_ref)],
                        Some(repo_path),
                    )
                    .await
                    .ok()
                    .and_then(|s| s.parse().ok());

                let behind_output = self
                    .execute_git_command_checked(
                        &["rev-list", "--count", &format!("HEAD..{}", upstream_ref)],
                        Some(repo_path),
                    )
                    .await
                    .ok()
                    .and_then(|s| s.parse().ok());

                (ahead_output, behind_output)
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };

        let mut extra_info = HashMap::new();
        extra_info.insert("scm_type".to_string(), "git".to_string());
        
        if let Some(branch) = &current_branch {
            extra_info.insert("branch".to_string(), branch.clone());
        }

        Ok(StatusResult {
            current_revision,
            current_branch,
            has_changes,
            has_untracked,
            ahead_count,
            behind_count,
            extra_info,
        })
    }

    fn is_repository(&self, path: &Path) -> bool {
        path.join(".git").exists()
    }

    fn scm_type(&self) -> ScmType {
        ScmType::Git
    }

    async fn get_current_revision(&self, repo_path: &Path) -> Result<String, ScmError> {
        self.execute_git_command_checked(&["rev-parse", "HEAD"], Some(repo_path))
            .await
    }

    async fn has_changes(&self, repo_path: &Path) -> Result<bool, ScmError> {
        let output = self
            .execute_git_command_checked(&["status", "--porcelain"], Some(repo_path))
            .await?;
        Ok(!output.is_empty())
    }
}

impl AsAny for GitScm {
    fn as_any(&self) -> &dyn Any {
        self
    }
}