use super::git_scm::GitScm;
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
        } else {
            None
        }
    }

    /// Check if an SCM type is available on the system
    pub async fn check_scm_availability(scm_type: ScmType) -> Result<bool, ScmError> {
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
        }
    }

    /// Get all available SCM types on the system
    pub async fn get_available_scm_types() -> Vec<ScmType> {
        let mut available = Vec::new();
        
        for scm_type in [ScmType::Git, ScmType::Svn, ScmType::P4] {
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
    }

    #[test]
    fn test_detect_scm_type() {
        use std::path::Path;
        
        // This test would need actual repository directories to work properly
        // For now, we just test the logic with non-existent paths
        let non_existent_path = Path::new("/non/existent/path");
        assert_eq!(ScmFactory::detect_scm_type(non_existent_path), None);
    }
}