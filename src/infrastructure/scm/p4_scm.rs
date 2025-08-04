use super::scm_interface::{AsAny, CloneOptions, ScmError, ScmOperations, StatusResult, SyncOptions};
use crate::domain::value_objects::scm_type::ScmType;
use async_trait::async_trait;
use std::any::Any;
use std::collections::HashMap;
use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;

/// Perforce (P4) implementation of SCM operations
pub struct P4Scm {
    p4_executable: String,
}

impl Default for P4Scm {
    fn default() -> Self {
        Self {
            p4_executable: "p4".to_string(),
        }
    }
}

impl P4Scm {
    /// Create a new P4 SCM instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new P4 SCM instance with custom executable path
    pub fn with_executable(executable: impl Into<String>) -> Self {
        Self {
            p4_executable: executable.into(),
        }
    }

    /// Check if p4 executable is available
    pub async fn check_availability(&self) -> Result<(), ScmError> {
        let output = Command::new(&self.p4_executable)
            .arg("info")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await?;

        if !output.status.success() {
            return Err(ScmError::executable_not_found(&self.p4_executable));
        }

        Ok(())
    }

    /// Execute a P4 command in the given directory
    async fn execute_p4_command(
        &self,
        args: &[&str],
        working_dir: Option<&Path>,
        env_vars: Option<&HashMap<String, String>>,
    ) -> Result<std::process::Output, ScmError> {
        let mut cmd = Command::new(&self.p4_executable);
        cmd.args(args).stdout(Stdio::piped()).stderr(Stdio::piped());

        if let Some(dir) = working_dir {
            cmd.current_dir(dir);
        }

        // Set P4 environment variables if provided
        if let Some(env) = env_vars {
            for (key, value) in env {
                cmd.env(key, value);
            }
        }

        let output = cmd.output().await?;
        Ok(output)
    }

    /// Execute a P4 command and check for success
    async fn execute_p4_command_checked(
        &self,
        args: &[&str],
        working_dir: Option<&Path>,
        env_vars: Option<&HashMap<String, String>>,
    ) -> Result<String, ScmError> {
        let output = self.execute_p4_command(args, working_dir, env_vars).await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let command = format!("{} {}", self.p4_executable, args.join(" "));
            return Err(ScmError::command_failed(
                command,
                output.status.code().unwrap_or(-1),
                stderr,
            ));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Parse P4 URL to extract server and depot information
    fn parse_p4_url(&self, url: &str) -> Result<(String, String), ScmError> {
        // P4 URLs can be: perforce://server:port//depot/path or server:port//depot/path
        let url = url.trim_start_matches("perforce://").trim_start_matches("p4://");
        
        if let Some(depot_pos) = url.find("//") {
            let server = url[..depot_pos].to_string();
            let depot_path = url[depot_pos..].to_string();
            Ok((server, depot_path))
        } else {
            Err(ScmError::InvalidUrl { url: url.to_string() })
        }
    }

    /// Build P4 environment variables
    fn build_p4_env(
        &self,
        server: &str,
        username: &Option<String>,
        password: &Option<String>,
        client: Option<&str>,
    ) -> HashMap<String, String> {
        let mut env = HashMap::new();
        
        env.insert("P4PORT".to_string(), server.to_string());
        
        if let Some(user) = username {
            env.insert("P4USER".to_string(), user.clone());
        }
        
        if let Some(pass) = password {
            env.insert("P4PASSWD".to_string(), pass.clone());
        }
        
        if let Some(client_name) = client {
            env.insert("P4CLIENT".to_string(), client_name.to_string());
        }
        
        env
    }

    /// Create a P4 client workspace specification
    async fn create_client_workspace(
        &self,
        client_name: &str,
        root_path: &Path,
        depot_path: &str,
        env_vars: &HashMap<String, String>,
    ) -> Result<(), ScmError> {
        // Create client spec content
        let client_spec = format!(
            "Client: {}\nRoot: {}\nView:\n\t{}/... //{}/...\n",
            client_name,
            root_path.to_string_lossy(),
            depot_path,
            client_name
        );

        // Write client spec to temporary file
        let temp_file = std::env::temp_dir().join(format!("{}.client", client_name));
        tokio::fs::write(&temp_file, client_spec).await?;

        // Load client spec
        let result = self
            .execute_p4_command_checked(
                &["client", "-i", temp_file.to_str().unwrap()],
                None,
                Some(env_vars),
            )
            .await;

        // Clean up temp file
        let _ = tokio::fs::remove_file(&temp_file).await;

        result.map(|_| ())
    }
}

#[async_trait]
impl ScmOperations for P4Scm {
    async fn clone_repository(
        &self,
        url: &str,
        dest_path: &Path,
        options: &CloneOptions,
    ) -> Result<(), ScmError> {
        let (server, depot_path) = self.parse_p4_url(url)?;
        
        // Generate client workspace name
        let client_name = format!(
            "wmgr-{}-{}",
            dest_path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("workspace"),
            std::process::id()
        );

        // Build environment variables
        let env_vars = self.build_p4_env(
            &server,
            &options.username,
            &options.password,
            Some(&client_name),
        );

        // Create destination directory
        tokio::fs::create_dir_all(dest_path).await?;

        // Create client workspace
        self.create_client_workspace(&client_name, dest_path, &depot_path, &env_vars)
            .await
            .map_err(|e| ScmError::clone_failed(format!("Failed to create P4 client: {}", e)))?;

        // Sync files
        let mut sync_args = vec!["sync"];
        
        let sync_path = if let Some(revision) = &options.revision {
            format!("{}@{}", depot_path, revision)
        } else {
            depot_path.to_string()
        };
        sync_args.push(&sync_path);

        self.execute_p4_command_checked(&sync_args, Some(dest_path), Some(&env_vars))
            .await
            .map_err(|e| ScmError::clone_failed(format!("P4 sync failed: {}", e)))?;

        // Store P4 configuration in .p4 directory
        let p4_dir = dest_path.join(".p4");
        tokio::fs::create_dir_all(&p4_dir).await?;
        
        let config = format!(
            "P4PORT={}\nP4CLIENT={}\nP4USER={}\nDEPOT_PATH={}\n",
            server,
            client_name,
            options.username.as_deref().unwrap_or(""),
            depot_path
        );
        
        tokio::fs::write(p4_dir.join("config"), config).await?;

        Ok(())
    }

    async fn sync_repository(
        &self,
        repo_path: &Path,
        options: &SyncOptions,
    ) -> Result<(), ScmError> {
        // Read P4 configuration
        let config_path = repo_path.join(".p4").join("config");
        if !config_path.exists() {
            return Err(ScmError::InvalidRepository {
                scm_type: ScmType::P4,
                path: repo_path.to_string_lossy().to_string(),
            });
        }

        let config_content = tokio::fs::read_to_string(&config_path).await?;
        let mut p4_config = HashMap::new();
        
        for line in config_content.lines() {
            if let Some((key, value)) = line.split_once('=') {
                p4_config.insert(key.to_string(), value.to_string());
            }
        }

        let server = p4_config.get("P4PORT").ok_or_else(|| ScmError::Internal {
            message: "P4PORT not found in config".to_string(),
        })?;
        
        let client = p4_config.get("P4CLIENT").ok_or_else(|| ScmError::Internal {
            message: "P4CLIENT not found in config".to_string(),
        })?;
        
        let depot_path = p4_config.get("DEPOT_PATH").ok_or_else(|| ScmError::Internal {
            message: "DEPOT_PATH not found in config".to_string(),
        })?;

        // Build environment variables
        let username = options.username.as_ref().or_else(|| p4_config.get("P4USER"));
        let env_vars = self.build_p4_env(
            server,
            &username.cloned(),
            &options.password,
            Some(client),
        );

        // Handle force option - revert changes
        if options.force {
            self.execute_p4_command_checked(
                &["revert", "..."],
                Some(repo_path),
                Some(&env_vars),
            )
            .await
            .map_err(|e| ScmError::sync_failed(format!("P4 revert failed: {}", e)))?;
        }

        // Sync to latest or specific revision
        let mut sync_args = vec!["sync"];
        
        let sync_path = if let Some(revision) = &options.revision {
            format!("{}@{}", depot_path, revision)
        } else {
            depot_path.to_string()
        };
        sync_args.push(&sync_path);

        self.execute_p4_command_checked(&sync_args, Some(repo_path), Some(&env_vars))
            .await
            .map_err(|e| ScmError::sync_failed(format!("P4 sync failed: {}", e)))?;

        Ok(())
    }

    async fn get_status(&self, repo_path: &Path) -> Result<StatusResult, ScmError> {
        // Read P4 configuration
        let config_path = repo_path.join(".p4").join("config");
        if !config_path.exists() {
            return Err(ScmError::InvalidRepository {
                scm_type: ScmType::P4,
                path: repo_path.to_string_lossy().to_string(),
            });
        }

        let config_content = tokio::fs::read_to_string(&config_path).await?;
        let mut p4_config = HashMap::new();
        
        for line in config_content.lines() {
            if let Some((key, value)) = line.split_once('=') {
                p4_config.insert(key.to_string(), value.to_string());
            }
        }

        let server = p4_config.get("P4PORT").unwrap();
        let client = p4_config.get("P4CLIENT").unwrap();
        let depot_path = p4_config.get("DEPOT_PATH").unwrap();

        let env_vars = self.build_p4_env(
            server,
            &p4_config.get("P4USER").cloned(),
            &None,
            Some(client),
        );

        // Get current changelist
        let current_revision = self
            .execute_p4_command_checked(
                &["changes", "-m1", &format!("{}#have", depot_path)],
                Some(repo_path),
                Some(&env_vars),
            )
            .await
            .map_err(|e| ScmError::status_failed(format!("Failed to get changelist: {}", e)))?
            .split_whitespace()
            .nth(1)
            .unwrap_or("unknown")
            .to_string();

        // Check for opened files (pending changes)
        let opened_output = self
            .execute_p4_command_checked(&["opened"], Some(repo_path), Some(&env_vars))
            .await
            .unwrap_or_default();

        let has_changes = !opened_output.is_empty();

        // P4 doesn't have untracked files concept in the same way
        let has_untracked = false;

        let mut extra_info = HashMap::new();
        extra_info.insert("scm_type".to_string(), "p4".to_string());
        extra_info.insert("server".to_string(), server.clone());
        extra_info.insert("client".to_string(), client.clone());
        extra_info.insert("depot_path".to_string(), depot_path.clone());

        Ok(StatusResult {
            current_revision,
            current_branch: Some(depot_path.clone()), // Use depot path as "branch"
            has_changes,
            has_untracked,
            ahead_count: None, // P4 doesn't have ahead/behind concept
            behind_count: None,
            extra_info,
        })
    }

    fn is_repository(&self, path: &Path) -> bool {
        path.join(".p4").exists() && path.join(".p4").join("config").exists()
    }

    fn scm_type(&self) -> ScmType {
        ScmType::P4
    }

    async fn get_current_revision(&self, repo_path: &Path) -> Result<String, ScmError> {
        let status = self.get_status(repo_path).await?;
        Ok(status.current_revision)
    }

    async fn has_changes(&self, repo_path: &Path) -> Result<bool, ScmError> {
        let status = self.get_status(repo_path).await?;
        Ok(status.has_changes)
    }
}

impl AsAny for P4Scm {
    fn as_any(&self) -> &dyn Any {
        self
    }
}