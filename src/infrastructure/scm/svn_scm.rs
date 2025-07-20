use super::scm_interface::{AsAny, CloneOptions, ScmError, ScmOperations, StatusResult, SyncOptions};
use crate::domain::value_objects::scm_type::ScmType;
use async_trait::async_trait;
use std::any::Any;
use std::collections::HashMap;
use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;

/// SVN (Subversion) implementation of SCM operations
pub struct SvnScm {
    svn_executable: String,
}

impl Default for SvnScm {
    fn default() -> Self {
        Self {
            svn_executable: "svn".to_string(),
        }
    }
}

impl SvnScm {
    /// Create a new SVN SCM instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new SVN SCM instance with custom executable path
    pub fn with_executable(executable: impl Into<String>) -> Self {
        Self {
            svn_executable: executable.into(),
        }
    }

    /// Check if svn executable is available
    pub async fn check_availability(&self) -> Result<(), ScmError> {
        let output = Command::new(&self.svn_executable)
            .arg("--version")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        if !output.status.success() {
            return Err(ScmError::executable_not_found(&self.svn_executable));
        }

        Ok(())
    }

    /// Execute an SVN command in the given directory
    async fn execute_svn_command(
        &self,
        args: &[&str],
        working_dir: Option<&Path>,
    ) -> Result<std::process::Output, ScmError> {
        let mut cmd = Command::new(&self.svn_executable);
        cmd.args(args).stdout(Stdio::piped()).stderr(Stdio::piped());

        if let Some(dir) = working_dir {
            cmd.current_dir(dir);
        }

        let output = cmd.output().await?;
        Ok(output)
    }

    /// Execute an SVN command and check for success
    async fn execute_svn_command_checked(
        &self,
        args: &[&str],
        working_dir: Option<&Path>,
    ) -> Result<String, ScmError> {
        let output = self.execute_svn_command(args, working_dir).await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let command = format!("{} {}", self.svn_executable, args.join(" "));
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
            args.push("--username".to_string());
            args.push(user.clone());
        }
        
        if let Some(pass) = password {
            args.push("--password".to_string());
            args.push(pass.clone());
        }
        
        // Add non-interactive flag to prevent prompts
        args.push("--non-interactive".to_string());
        
        args
    }
}

#[async_trait]
impl ScmOperations for SvnScm {
    async fn clone_repository(
        &self,
        url: &str,
        dest_path: &Path,
        options: &CloneOptions,
    ) -> Result<(), ScmError> {
        let mut args = vec!["checkout"];

        // Add authentication if provided
        let auth_args = self.build_auth_args(&options.username, &options.password);
        let auth_str_args: Vec<&str> = auth_args.iter().map(|s| s.as_str()).collect();
        args.extend(auth_str_args);

        // Add revision if specified
        if let Some(revision) = &options.revision {
            args.push("--revision");
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

        let output = self.execute_svn_command(&args, None).await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ScmError::clone_failed(format!(
                "SVN checkout failed: {}",
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
        let mut args = vec!["update"];

        // Add authentication if provided
        let auth_args = self.build_auth_args(&options.username, &options.password);
        let auth_str_args: Vec<&str> = auth_args.iter().map(|s| s.as_str()).collect();
        args.extend(auth_str_args);

        // Add revision if specified
        if let Some(revision) = &options.revision {
            args.push("--revision");
            args.push(revision);
        }

        // Handle force option - revert local changes first
        if options.force {
            let revert_args = vec!["revert", "--recursive", "."];
            self.execute_svn_command_checked(&revert_args, Some(repo_path))
                .await
                .map_err(|e| ScmError::sync_failed(format!("Revert failed: {}", e)))?;
        }

        // Add extra options
        for option in &options.extra_options {
            args.push(option);
        }

        let output = self.execute_svn_command(&args, Some(repo_path)).await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ScmError::sync_failed(format!(
                "SVN update failed: {}",
                stderr
            )));
        }

        Ok(())
    }

    async fn get_status(&self, repo_path: &Path) -> Result<StatusResult, ScmError> {
        // Get current revision using svn info
        let info_output = self
            .execute_svn_command_checked(&["info", "--show-item", "revision"], Some(repo_path))
            .await
            .map_err(|e| ScmError::status_failed(format!("Failed to get revision: {}", e)))?;

        let current_revision = info_output.trim().to_string();

        // Get URL to extract branch-like information
        let url_output = self
            .execute_svn_command_checked(&["info", "--show-item", "url"], Some(repo_path))
            .await
            .ok();

        // Extract branch-like info from URL (trunk, branches/name, tags/name)
        let current_branch = url_output.as_ref().and_then(|url| {
            if url.contains("/trunk") {
                Some("trunk".to_string())
            } else if let Some(branches_pos) = url.find("/branches/") {
                let after_branches = &url[branches_pos + 10..];
                after_branches.split('/').next().map(|s| format!("branches/{}", s))
            } else if let Some(tags_pos) = url.find("/tags/") {
                let after_tags = &url[tags_pos + 6..];
                after_tags.split('/').next().map(|s| format!("tags/{}", s))
            } else {
                None
            }
        });

        // Check for local modifications
        let status_output = self
            .execute_svn_command_checked(&["status"], Some(repo_path))
            .await
            .map_err(|e| ScmError::status_failed(format!("Failed to get status: {}", e)))?;

        let has_changes = status_output
            .lines()
            .any(|line| !line.trim().is_empty() && line.chars().next().unwrap_or(' ') != '?');

        let has_untracked = status_output
            .lines()
            .any(|line| line.chars().next().unwrap_or(' ') == '?');

        let mut extra_info = HashMap::new();
        extra_info.insert("scm_type".to_string(), "svn".to_string());
        
        if let Some(url) = url_output {
            extra_info.insert("url".to_string(), url);
        }
        
        if let Some(branch) = &current_branch {
            extra_info.insert("branch_path".to_string(), branch.clone());
        }

        Ok(StatusResult {
            current_revision,
            current_branch,
            has_changes,
            has_untracked,
            ahead_count: None, // SVN doesn't have ahead/behind concept
            behind_count: None,
            extra_info,
        })
    }

    fn is_repository(&self, path: &Path) -> bool {
        path.join(".svn").exists()
    }

    fn scm_type(&self) -> ScmType {
        ScmType::Svn
    }

    async fn get_current_revision(&self, repo_path: &Path) -> Result<String, ScmError> {
        self.execute_svn_command_checked(&["info", "--show-item", "revision"], Some(repo_path))
            .await
    }

    async fn has_changes(&self, repo_path: &Path) -> Result<bool, ScmError> {
        let output = self
            .execute_svn_command_checked(&["status"], Some(repo_path))
            .await?;
        
        // Check if there are any modified files (not untracked)
        let has_changes = output
            .lines()
            .any(|line| !line.trim().is_empty() && line.chars().next().unwrap_or(' ') != '?');
        
        Ok(has_changes)
    }
}

impl AsAny for SvnScm {
    fn as_any(&self) -> &dyn Any {
        self
    }
}