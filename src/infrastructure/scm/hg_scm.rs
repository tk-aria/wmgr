use super::scm_interface::{AsAny, CloneOptions, ScmError, ScmOperations, StatusResult, SyncOptions};
use crate::domain::value_objects::scm_type::ScmType;
use async_trait::async_trait;
use std::any::Any;
use std::collections::HashMap;
use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;

/// Mercurial (Hg) implementation of SCM operations
pub struct HgScm {
    hg_executable: String,
}

impl Default for HgScm {
    fn default() -> Self {
        Self {
            hg_executable: "hg".to_string(),
        }
    }
}

impl HgScm {
    /// Create a new Mercurial SCM instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new Mercurial SCM instance with custom executable path
    pub fn with_executable(executable: impl Into<String>) -> Self {
        Self {
            hg_executable: executable.into(),
        }
    }

    /// Check if hg executable is available
    pub async fn check_availability(&self) -> Result<(), ScmError> {
        let output = Command::new(&self.hg_executable)
            .arg("--version")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        if !output.status.success() {
            return Err(ScmError::executable_not_found(&self.hg_executable));
        }

        Ok(())
    }

    /// Execute an hg command in the given directory
    async fn execute_hg_command(
        &self,
        args: &[&str],
        working_dir: Option<&Path>,
    ) -> Result<std::process::Output, ScmError> {
        let mut cmd = Command::new(&self.hg_executable);
        cmd.args(args).stdout(Stdio::piped()).stderr(Stdio::piped());

        if let Some(dir) = working_dir {
            cmd.current_dir(dir);
        }

        let output = cmd.output().await?;
        Ok(output)
    }

    /// Execute an hg command and check for success
    async fn execute_hg_command_checked(
        &self,
        args: &[&str],
        working_dir: Option<&Path>,
    ) -> Result<String, ScmError> {
        let output = self.execute_hg_command(args, working_dir).await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let command = format!("{} {}", self.hg_executable, args.join(" "));
            return Err(ScmError::command_failed(
                command,
                output.status.code().unwrap_or(-1),
                stderr,
            ));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Build authentication arguments if credentials are provided
    fn build_auth_args(&self, username: &Option<String>, password: &Option<String>) -> Vec<String> {
        let mut args = Vec::new();
        
        if let Some(user) = username {
            args.push("--config".to_string());
            args.push(format!("auth.x.username={}", user));
        }
        
        if let Some(pass) = password {
            args.push("--config".to_string());
            args.push(format!("auth.x.password={}", pass));
        }
        
        args
    }
}

#[async_trait]
impl ScmOperations for HgScm {
    async fn clone_repository(
        &self,
        url: &str,
        dest_path: &Path,
        options: &CloneOptions,
    ) -> Result<(), ScmError> {
        let mut args = vec!["clone"];

        // Add authentication if provided
        let auth_args = self.build_auth_args(&options.username, &options.password);
        let auth_str_args: Vec<&str> = auth_args.iter().map(|s| s.as_str()).collect();
        args.extend(auth_str_args);

        // Add branch option (use --branch for Mercurial)
        if let Some(branch) = &options.branch {
            args.push("--branch");
            args.push(branch);
        }

        // Add revision option (use --rev for Mercurial)
        if let Some(revision) = &options.revision {
            args.push("--rev");
            args.push(revision);
        }

        // Add extra options
        for option in &options.extra_options {
            args.push(option);
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

        let output = self.execute_hg_command(&args, None).await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ScmError::clone_failed(format!(
                "Mercurial clone failed: {}",
                stderr
            )));
        }

        Ok(())
    }

    async fn sync_repository(
        &self,
        repo_path: &Path,
        options: &SyncOptions,
    ) -> Result<(), ScmError> {
        let mut pull_args = vec!["pull"];

        // Add authentication if provided
        let auth_args = self.build_auth_args(&options.username, &options.password);
        let auth_str_args: Vec<&str> = auth_args.iter().map(|s| s.as_str()).collect();
        pull_args.extend(auth_str_args);

        // Add branch option for pull
        if let Some(branch) = &options.branch {
            pull_args.push("--branch");
            pull_args.push(branch);
        }

        // Add revision option for pull
        if let Some(revision) = &options.revision {
            pull_args.push("--rev");
            pull_args.push(revision);
        }

        // Add extra options for pull
        for option in &options.extra_options {
            pull_args.push(option);
        }

        // Pull latest changes
        self.execute_hg_command_checked(&pull_args, Some(repo_path))
            .await
            .map_err(|e| ScmError::sync_failed(format!("Pull failed: {}", e)))?;

        // Handle force option
        if options.force {
            // Update with --clean to discard local changes
            let mut update_args = vec!["update", "--clean"];
            
            if let Some(revision) = &options.revision {
                update_args.push("--rev");
                update_args.push(revision);
            } else if let Some(branch) = &options.branch {
                update_args.push("--rev");
                update_args.push(branch);
            }

            self.execute_hg_command_checked(&update_args, Some(repo_path))
                .await
                .map_err(|e| ScmError::sync_failed(format!("Force update failed: {}", e)))?;
        } else {
            // Normal update
            let mut update_args = vec!["update"];
            
            if let Some(revision) = &options.revision {
                update_args.push("--rev");
                update_args.push(revision);
            } else if let Some(branch) = &options.branch {
                update_args.push("--rev");
                update_args.push(branch);
            }

            self.execute_hg_command_checked(&update_args, Some(repo_path))
                .await
                .map_err(|e| ScmError::sync_failed(format!("Update failed: {}", e)))?;
        }

        Ok(())
    }

    async fn get_status(&self, repo_path: &Path) -> Result<StatusResult, ScmError> {
        // Get current revision (changeset hash)
        let current_revision = self
            .execute_hg_command_checked(&["identify", "--id"], Some(repo_path))
            .await
            .map_err(|e| ScmError::status_failed(format!("Failed to get revision: {}", e)))?
            .trim_end_matches('+') // Remove '+' indicating uncommitted changes
            .to_string();

        // Get current branch
        let current_branch = self
            .execute_hg_command_checked(&["branch"], Some(repo_path))
            .await
            .ok()
            .filter(|b| !b.is_empty());

        // Check for uncommitted changes using hg status
        let status_output = self
            .execute_hg_command_checked(&["status"], Some(repo_path))
            .await
            .map_err(|e| ScmError::status_failed(format!("Failed to get status: {}", e)))?;

        // Parse status output
        let has_changes = status_output
            .lines()
            .any(|line| {
                let status_char = line.chars().next().unwrap_or(' ');
                matches!(status_char, 'M' | 'A' | 'R' | '!')
            });

        let has_untracked = status_output
            .lines()
            .any(|line| line.chars().next().unwrap_or(' ') == '?');

        // Get summary info for additional details
        let summary_output = self
            .execute_hg_command_checked(&["summary"], Some(repo_path))
            .await
            .ok();

        // For Mercurial, we can get ahead/behind info by comparing with default path
        let (ahead_count, behind_count) = self.get_ahead_behind_count(repo_path).await;

        let mut extra_info = HashMap::new();
        extra_info.insert("scm_type".to_string(), "hg".to_string());
        
        if let Some(branch) = &current_branch {
            extra_info.insert("branch".to_string(), branch.clone());
        }

        // Add bookmark information if available
        if let Ok(bookmark_output) = self
            .execute_hg_command_checked(&["bookmarks"], Some(repo_path))
            .await
        {
            let active_bookmark = bookmark_output
                .lines()
                .find(|line| line.contains(" * "))
                .and_then(|line| line.split_whitespace().nth(1))
                .map(|bookmark| bookmark.to_string());
            
            if let Some(bookmark) = active_bookmark {
                extra_info.insert("active_bookmark".to_string(), bookmark);
            }
        }

        // Add phase information
        if let Ok(phase_output) = self
            .execute_hg_command_checked(&["phase", "."], Some(repo_path))
            .await
        {
            if let Some(phase) = phase_output.split(':').nth(1) {
                extra_info.insert("phase".to_string(), phase.trim().to_string());
            }
        }

        if let Some(summary) = summary_output {
            extra_info.insert("summary".to_string(), summary);
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
        path.join(".hg").exists()
    }

    fn scm_type(&self) -> ScmType {
        ScmType::Hg
    }

    async fn get_current_revision(&self, repo_path: &Path) -> Result<String, ScmError> {
        let output = self
            .execute_hg_command_checked(&["identify", "--id"], Some(repo_path))
            .await?;
        
        // Remove '+' suffix that indicates uncommitted changes
        Ok(output.trim_end_matches('+').to_string())
    }

    async fn has_changes(&self, repo_path: &Path) -> Result<bool, ScmError> {
        let output = self
            .execute_hg_command_checked(&["status"], Some(repo_path))
            .await?;
        
        // Check if there are any modified, added, removed files (not untracked)
        let has_changes = output
            .lines()
            .any(|line| {
                let status_char = line.chars().next().unwrap_or(' ');
                matches!(status_char, 'M' | 'A' | 'R' | '!')
            });
        
        Ok(has_changes)
    }
}

impl HgScm {
    /// Get ahead/behind count compared to default remote path
    async fn get_ahead_behind_count(&self, repo_path: &Path) -> (Option<usize>, Option<usize>) {
        // Get default path (remote repository)
        let default_path = self
            .execute_hg_command_checked(&["paths", "default"], Some(repo_path))
            .await
            .ok();

        if default_path.is_none() {
            return (None, None);
        }

        // Get current branch
        let current_branch = self
            .execute_hg_command_checked(&["branch"], Some(repo_path))
            .await
            .ok();

        if let Some(branch) = current_branch {
            // Get outgoing changesets (ahead)
            let ahead_count = self
                .execute_hg_command(&["outgoing", "--template", "x", "--branch", &branch], Some(repo_path))
                .await
                .ok()
                .and_then(|output| {
                    if output.status.success() {
                        Some(String::from_utf8_lossy(&output.stdout).chars().count())
                    } else {
                        None
                    }
                });

            // Get incoming changesets (behind)
            let behind_count = self
                .execute_hg_command(&["incoming", "--template", "x", "--branch", &branch], Some(repo_path))
                .await
                .ok()
                .and_then(|output| {
                    if output.status.success() {
                        Some(String::from_utf8_lossy(&output.stdout).chars().count())
                    } else {
                        None
                    }
                });

            (ahead_count, behind_count)
        } else {
            (None, None)
        }
    }
}

impl AsAny for HgScm {
    fn as_any(&self) -> &dyn Any {
        self
    }
}