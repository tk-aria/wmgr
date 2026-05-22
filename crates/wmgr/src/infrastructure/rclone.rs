use std::path::{Path, PathBuf};
use tokio::process::Command;

pub struct RcloneManager {
    config_path: PathBuf,
}

#[derive(Debug)]
pub enum RcloneError {
    NotInstalled,
    AuthRequired(String),
    CommandFailed(String),
    IoError(String),
}

impl std::fmt::Display for RcloneError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RcloneError::NotInstalled => write!(f, "rclone is not installed. Install it from https://rclone.org/install/"),
            RcloneError::AuthRequired(remote) => write!(f, "Google Drive authentication required for remote '{}'", remote),
            RcloneError::CommandFailed(msg) => write!(f, "rclone command failed: {}", msg),
            RcloneError::IoError(msg) => write!(f, "rclone I/O error: {}", msg),
        }
    }
}

impl std::error::Error for RcloneError {}

impl RcloneManager {
    pub fn new() -> Self {
        let config_path = Self::default_config_path();
        Self { config_path }
    }

    pub fn with_config_path(config_path: PathBuf) -> Self {
        Self { config_path }
    }

    fn default_config_path() -> PathBuf {
        std::env::var("HOME")
            .map(|h| PathBuf::from(h).join(".config/wmgr/rclone.conf"))
            .unwrap_or_else(|_| PathBuf::from("/tmp/wmgr-rclone.conf"))
    }

    pub fn config_path(&self) -> &Path {
        &self.config_path
    }

    pub async fn check_installed() -> bool {
        Command::new("rclone")
            .arg("--version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .await
            .map(|s| s.success())
            .unwrap_or(false)
    }

    pub async fn ensure_remote(&self, remote_name: &str) -> Result<(), RcloneError> {
        if self.remote_exists(remote_name).await? {
            if self.check_remote_auth(remote_name).await? {
                return Ok(());
            }
            eprintln!("Google Drive remote '{}' exists but authentication is expired or invalid.", remote_name);
        } else {
            self.create_remote(remote_name).await?;
        }

        self.authorize_remote(remote_name).await
    }

    async fn remote_exists(&self, remote_name: &str) -> Result<bool, RcloneError> {
        let output = Command::new("rclone")
            .args(["listremotes", "--config", &self.config_path.display().to_string()])
            .output()
            .await
            .map_err(|e| RcloneError::IoError(e.to_string()))?;

        let remotes = String::from_utf8_lossy(&output.stdout);
        let target = format!("{}:", remote_name);
        Ok(remotes.lines().any(|line| line.trim() == target))
    }

    async fn check_remote_auth(&self, remote_name: &str) -> Result<bool, RcloneError> {
        let output = Command::new("rclone")
            .args([
                "lsf",
                &format!("{}:", remote_name),
                "--max-depth", "0",
                "--config", &self.config_path.display().to_string(),
            ])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output()
            .await
            .map_err(|e| RcloneError::IoError(e.to_string()))?;

        Ok(output.status.success())
    }

    async fn create_remote(&self, remote_name: &str) -> Result<(), RcloneError> {
        if let Some(parent) = self.config_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| RcloneError::IoError(e.to_string()))?;
        }

        let config_content = format!("[{}]\ntype = drive\n", remote_name);
        tokio::fs::write(&self.config_path, config_content)
            .await
            .map_err(|e| RcloneError::IoError(format!("Failed to write rclone config: {}", e)))?;

        Ok(())
    }

    async fn authorize_remote(&self, remote_name: &str) -> Result<(), RcloneError> {
        if Self::is_ci() {
            return Err(RcloneError::AuthRequired(format!(
                "{}. Set WMGR_GDRIVE_TOKEN or run 'wmgr sync' locally first.",
                remote_name,
            )));
        }

        if Self::has_local_browser() {
            eprintln!("Google Drive authentication required for remote '{}'.", remote_name);
            eprintln!("A browser window will open for OAuth authorization...");
        } else {
            eprintln!("Google Drive authentication required for remote '{}'.", remote_name);
            eprintln!("No local browser detected — remote device authorization will be used.");
            eprintln!("A URL will be shown. Open it on any device with a browser.");
        }

        let output = Command::new("rclone")
            .args(["config", "reconnect", &format!("{}:", remote_name),
                   "--config", &self.config_path.display().to_string()])
            .stdin(std::process::Stdio::inherit())
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .status()
            .await
            .map_err(|e| RcloneError::IoError(e.to_string()))?;

        if !output.success() {
            return Err(RcloneError::CommandFailed("OAuth authorization failed or was cancelled.".to_string()));
        }

        eprintln!("Google Drive authentication successful for remote '{}'.", remote_name);
        Ok(())
    }

    fn is_ci() -> bool {
        std::env::var("CI").is_ok()
            || std::env::var("GITHUB_ACTIONS").is_ok()
    }

    fn has_local_browser() -> bool {
        if std::env::var("WMGR_HEADLESS").is_ok() {
            return false;
        }
        if std::env::var("SSH_CONNECTION").is_ok() || std::env::var("SSH_TTY").is_ok() {
            return false;
        }
        #[cfg(target_os = "linux")]
        {
            return std::env::var("DISPLAY").is_ok() || std::env::var("WAYLAND_DISPLAY").is_ok();
        }
        #[cfg(not(target_os = "linux"))]
        {
            true
        }
    }

    pub async fn copy(
        &self,
        source: &str,
        dest: &Path,
    ) -> Result<std::process::Output, RcloneError> {
        let output = Command::new("rclone")
            .args([
                "copy", source, &dest.display().to_string(),
                "--config", &self.config_path.display().to_string(),
            ])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output()
            .await
            .map_err(|e| RcloneError::IoError(e.to_string()))?;

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_path() {
        let manager = RcloneManager::new();
        let path = manager.config_path();
        assert!(path.to_string_lossy().contains("wmgr/rclone.conf"));
    }

    #[test]
    fn test_custom_config_path() {
        let manager = RcloneManager::with_config_path(PathBuf::from("/tmp/test-rclone.conf"));
        assert_eq!(manager.config_path(), Path::new("/tmp/test-rclone.conf"));
    }

    #[test]
    fn test_is_ci() {
        std::env::remove_var("CI");
        std::env::remove_var("GITHUB_ACTIONS");
        assert!(!RcloneManager::is_ci());
    }

    #[test]
    fn test_has_local_browser_headless_override() {
        std::env::set_var("WMGR_HEADLESS", "1");
        assert!(!RcloneManager::has_local_browser());
        std::env::remove_var("WMGR_HEADLESS");
    }
}
