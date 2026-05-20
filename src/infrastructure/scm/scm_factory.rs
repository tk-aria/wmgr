use super::git_scm::GitScm;
use super::hg_scm::HgScm;
use super::p4_scm::P4Scm;
use super::scm_interface::{ScmError, ScmOperations};
use super::svn_scm::SvnScm;
use crate::domain::value_objects::scm_type::ScmType;
use std::sync::Arc;

/// Factory for creating SCM implementation instances
pub struct ScmFactory;

impl ScmFactory {
    /// Create an SCM operations instance for the given SCM type
    pub fn create_scm(scm_type: ScmType) -> Result<Arc<dyn ScmOperations>, ScmError> {
        match scm_type {
            ScmType::Git => Ok(Arc::new(GitScm::new())),
            ScmType::Svn => Ok(Arc::new(SvnScm::new())),
            ScmType::P4 => Ok(Arc::new(P4Scm::new())),
            ScmType::Hg => Ok(Arc::new(HgScm::new())),
            ScmType::Http => Err(ScmError::UnsupportedOperation {
                scm_type: ScmType::Http,
                operation: "create_scm: HTTP downloads are handled directly, not via SCM interface".to_string(),
            }),
            ScmType::Symlink => Err(ScmError::UnsupportedOperation {
                scm_type: ScmType::Symlink,
                operation: "create_scm: Symlinks are handled directly, not via SCM interface".to_string(),
            }),
            ScmType::S3 => Err(ScmError::UnsupportedOperation {
                scm_type: ScmType::S3,
                operation: "create_scm: S3 downloads are handled directly, not via SCM interface".to_string(),
            }),
            ScmType::GDrive => Err(ScmError::UnsupportedOperation {
                scm_type: ScmType::GDrive,
                operation: "create_scm: Google Drive downloads are handled directly, not via SCM interface".to_string(),
            }),
        }
    }

    /// Create an SCM operations instance with custom executable path
    pub fn create_scm_with_executable(
        scm_type: ScmType,
        executable_path: &str,
    ) -> Result<Arc<dyn ScmOperations>, ScmError> {
        match scm_type {
            ScmType::Git => Ok(Arc::new(GitScm::with_executable(executable_path))),
            ScmType::Svn => Ok(Arc::new(SvnScm::with_executable(executable_path))),
            ScmType::P4 => Ok(Arc::new(P4Scm::with_executable(executable_path))),
            ScmType::Hg => Ok(Arc::new(HgScm::with_executable(executable_path))),
            ScmType::Http => Err(ScmError::UnsupportedOperation {
                scm_type: ScmType::Http,
                operation: "create_scm_with_executable: HTTP downloads do not use an executable".to_string(),
            }),
            ScmType::Symlink => Err(ScmError::UnsupportedOperation {
                scm_type: ScmType::Symlink,
                operation: "create_scm_with_executable: Symlinks do not use an executable".to_string(),
            }),
            ScmType::S3 => Err(ScmError::UnsupportedOperation {
                scm_type: ScmType::S3,
                operation: "create_scm_with_executable: S3 is handled directly".to_string(),
            }),
            ScmType::GDrive => Err(ScmError::UnsupportedOperation {
                scm_type: ScmType::GDrive,
                operation: "create_scm_with_executable: Google Drive is handled directly".to_string(),
            }),
        }
    }

    /// Detect SCM type from a repository path
    pub fn detect_scm_type(repo_path: &std::path::Path) -> Option<ScmType> {
        if repo_path.join(".git").exists() {
            Some(ScmType::Git)
        } else if repo_path.join(".svn").exists() {
            Some(ScmType::Svn)
        } else if repo_path.join(".p4").exists() {
            Some(ScmType::P4)
        } else if repo_path.join(".hg").exists() {
            Some(ScmType::Hg)
        } else {
            None
        }
    }

    /// Check if an SCM type is available on the system
    pub async fn check_scm_availability(scm_type: ScmType) -> Result<bool, ScmError> {
        match &scm_type {
            ScmType::Http | ScmType::Symlink => return Ok(true),
            ScmType::S3 => {
                return Self::check_command_availability("aws", &["--version"]).await;
            }
            ScmType::GDrive => {
                return Self::check_command_availability("rclone", &["--version"]).await;
            }
            _ => {}
        }

        let scm = Self::create_scm(scm_type.clone())?;

        match scm_type {
            ScmType::Git => {
                if let Some(git_scm) = scm.as_any().downcast_ref::<GitScm>() {
                    git_scm.check_availability().await.map(|_| true).or(Ok(false))
                } else {
                    Ok(false)
                }
            }
            ScmType::Svn => {
                if let Some(svn_scm) = scm.as_any().downcast_ref::<SvnScm>() {
                    svn_scm.check_availability().await.map(|_| true).or(Ok(false))
                } else {
                    Ok(false)
                }
            }
            ScmType::P4 => {
                if let Some(p4_scm) = scm.as_any().downcast_ref::<P4Scm>() {
                    p4_scm.check_availability().await.map(|_| true).or(Ok(false))
                } else {
                    Ok(false)
                }
            }
            ScmType::Hg => {
                if let Some(hg_scm) = scm.as_any().downcast_ref::<HgScm>() {
                    hg_scm.check_availability().await.map(|_| true).or(Ok(false))
                } else {
                    Ok(false)
                }
            }
            ScmType::Http | ScmType::Symlink | ScmType::S3 | ScmType::GDrive => unreachable!(),
        }
    }

    async fn check_command_availability(command: &str, args: &[&str]) -> Result<bool, ScmError> {
        use std::process::Stdio;
        use tokio::process::Command;

        let result = Command::new(command)
            .args(args)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await;

        match result {
            Ok(status) => Ok(status.success()),
            Err(_) => Ok(false),
        }
    }

    /// Get all available SCM types on the system
    pub async fn get_available_scm_types() -> Vec<ScmType> {
        let mut available = Vec::new();

        for scm_type in [ScmType::Git, ScmType::Svn, ScmType::P4, ScmType::Hg] {
            if Self::check_scm_availability(scm_type.clone()).await.unwrap_or(false) {
                available.push(scm_type);
            }
        }

        available
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_scm_instances() {
        let git_scm = ScmFactory::create_scm(ScmType::Git);
        assert!(git_scm.is_ok());
        assert_eq!(git_scm.unwrap().scm_type(), ScmType::Git);

        let svn_scm = ScmFactory::create_scm(ScmType::Svn);
        assert!(svn_scm.is_ok());
        assert_eq!(svn_scm.unwrap().scm_type(), ScmType::Svn);

        let p4_scm = ScmFactory::create_scm(ScmType::P4);
        assert!(p4_scm.is_ok());
        assert_eq!(p4_scm.unwrap().scm_type(), ScmType::P4);

        let hg_scm = ScmFactory::create_scm(ScmType::Hg);
        assert!(hg_scm.is_ok());
        assert_eq!(hg_scm.unwrap().scm_type(), ScmType::Hg);

        let http_scm = ScmFactory::create_scm(ScmType::Http);
        assert!(http_scm.is_err());

        let symlink_scm = ScmFactory::create_scm(ScmType::Symlink);
        assert!(symlink_scm.is_err());

        let s3_scm = ScmFactory::create_scm(ScmType::S3);
        assert!(s3_scm.is_err());

        let gdrive_scm = ScmFactory::create_scm(ScmType::GDrive);
        assert!(gdrive_scm.is_err());
    }

    #[test]
    fn test_detect_scm_type() {
        use std::path::Path;

        let non_existent_path = Path::new("/non/existent/path");
        assert_eq!(ScmFactory::detect_scm_type(non_existent_path), None);
    }
}
