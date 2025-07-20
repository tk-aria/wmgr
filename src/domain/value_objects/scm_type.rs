use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// SCM (Source Control Management) system type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScmType {
    /// Git version control system
    Git,
    /// Subversion (SVN) version control system
    Svn,
    /// Perforce (P4) version control system
    P4,
}

impl Default for ScmType {
    fn default() -> Self {
        Self::Git
    }
}

impl fmt::Display for ScmType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScmType::Git => write!(f, "git"),
            ScmType::Svn => write!(f, "svn"),
            ScmType::P4 => write!(f, "p4"),
        }
    }
}

impl FromStr for ScmType {
    type Err = ScmTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "git" => Ok(ScmType::Git),
            "svn" | "subversion" => Ok(ScmType::Svn),
            "p4" | "perforce" => Ok(ScmType::P4),
            _ => Err(ScmTypeError::UnsupportedScmType(s.to_string())),
        }
    }
}

impl ScmType {
    /// Check if this SCM type supports branches
    pub fn supports_branches(&self) -> bool {
        match self {
            ScmType::Git => true,
            ScmType::Svn => false, // SVN uses trunk/branches/tags structure
            ScmType::P4 => false,  // Perforce uses different branching model
        }
    }

    /// Check if this SCM type supports remotes
    pub fn supports_remotes(&self) -> bool {
        match self {
            ScmType::Git => true,
            ScmType::Svn => false, // SVN repositories are centralized
            ScmType::P4 => false,  // Perforce is centralized
        }
    }

    /// Check if this SCM type supports shallow clones
    pub fn supports_shallow_clone(&self) -> bool {
        match self {
            ScmType::Git => true,
            ScmType::Svn => false, // SVN doesn't have shallow clone concept
            ScmType::P4 => false,  // P4 sync is different from clone
        }
    }

    /// Get the typical file extensions for ignore files
    pub fn ignore_file_patterns(&self) -> Vec<&'static str> {
        match self {
            ScmType::Git => vec![".gitignore"],
            ScmType::Svn => vec![".svnignore"],
            ScmType::P4 => vec![".p4ignore"],
        }
    }

    /// Get the metadata directory name for this SCM
    pub fn metadata_dir(&self) -> &'static str {
        match self {
            ScmType::Git => ".git",
            ScmType::Svn => ".svn",
            ScmType::P4 => ".p4",
        }
    }

    /// Get the standard executable name for this SCM
    pub fn executable_name(&self) -> &'static str {
        match self {
            ScmType::Git => "git",
            ScmType::Svn => "svn",
            ScmType::P4 => "p4",
        }
    }

    /// Check if the URL scheme is appropriate for this SCM type
    pub fn is_valid_url_scheme(&self, url: &str) -> bool {
        match self {
            ScmType::Git => {
                url.starts_with("https://")
                    || url.starts_with("http://")
                    || url.starts_with("git://")
                    || url.starts_with("ssh://")
                    || url.starts_with("git@")
                    || url.starts_with("file://")
            }
            ScmType::Svn => {
                url.starts_with("https://")
                    || url.starts_with("http://")
                    || url.starts_with("svn://")
                    || url.starts_with("svn+ssh://")
                    || url.starts_with("file://")
            }
            ScmType::P4 => {
                url.starts_with("perforce://")
                    || url.starts_with("p4://")
                    || url.starts_with("ssl:")
                    || url.starts_with("tcp:")
                    || url.contains(":") // P4 server:port format
            }
        }
    }
}

/// Errors that can occur when working with SCM types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScmTypeError {
    /// The specified SCM type is not supported
    UnsupportedScmType(String),
    /// The URL scheme is not valid for the SCM type
    InvalidUrlScheme { scm: ScmType, url: String },
    /// Operation not supported by this SCM type
    UnsupportedOperation { scm: ScmType, operation: String },
}

impl fmt::Display for ScmTypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScmTypeError::UnsupportedScmType(scm) => {
                write!(f, "Unsupported SCM type: '{}'. Supported types are: git, svn, p4", scm)
            }
            ScmTypeError::InvalidUrlScheme { scm, url } => {
                write!(f, "Invalid URL scheme for {}: '{}'", scm, url)
            }
            ScmTypeError::UnsupportedOperation { scm, operation } => {
                write!(f, "Operation '{}' is not supported by {}", operation, scm)
            }
        }
    }
}

impl std::error::Error for ScmTypeError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scm_type_from_str() {
        assert_eq!("git".parse::<ScmType>().unwrap(), ScmType::Git);
        assert_eq!("svn".parse::<ScmType>().unwrap(), ScmType::Svn);
        assert_eq!("subversion".parse::<ScmType>().unwrap(), ScmType::Svn);
        assert_eq!("p4".parse::<ScmType>().unwrap(), ScmType::P4);
        assert_eq!("perforce".parse::<ScmType>().unwrap(), ScmType::P4);
        
        assert!("unknown".parse::<ScmType>().is_err());
    }

    #[test]
    fn test_scm_type_display() {
        assert_eq!(ScmType::Git.to_string(), "git");
        assert_eq!(ScmType::Svn.to_string(), "svn");
        assert_eq!(ScmType::P4.to_string(), "p4");
    }

    #[test]
    fn test_scm_capabilities() {
        assert!(ScmType::Git.supports_branches());
        assert!(!ScmType::Svn.supports_branches());
        assert!(!ScmType::P4.supports_branches());

        assert!(ScmType::Git.supports_remotes());
        assert!(!ScmType::Svn.supports_remotes());
        assert!(!ScmType::P4.supports_remotes());

        assert!(ScmType::Git.supports_shallow_clone());
        assert!(!ScmType::Svn.supports_shallow_clone());
        assert!(!ScmType::P4.supports_shallow_clone());
    }

    #[test]
    fn test_scm_metadata_dirs() {
        assert_eq!(ScmType::Git.metadata_dir(), ".git");
        assert_eq!(ScmType::Svn.metadata_dir(), ".svn");
        assert_eq!(ScmType::P4.metadata_dir(), ".p4");
    }

    #[test]
    fn test_url_scheme_validation() {
        // Git URLs
        assert!(ScmType::Git.is_valid_url_scheme("https://github.com/user/repo.git"));
        assert!(ScmType::Git.is_valid_url_scheme("git@github.com:user/repo.git"));
        assert!(ScmType::Git.is_valid_url_scheme("ssh://git@server/repo.git"));
        
        // SVN URLs
        assert!(ScmType::Svn.is_valid_url_scheme("https://svn.example.com/repo"));
        assert!(ScmType::Svn.is_valid_url_scheme("svn://server/repo"));
        assert!(ScmType::Svn.is_valid_url_scheme("svn+ssh://server/repo"));
        
        // P4 URLs
        assert!(ScmType::P4.is_valid_url_scheme("perforce://server:1666"));
        assert!(ScmType::P4.is_valid_url_scheme("p4://server:1666"));
        assert!(ScmType::P4.is_valid_url_scheme("ssl:server:1666"));
        assert!(ScmType::P4.is_valid_url_scheme("server:1666"));
    }

    #[test]
    fn test_serde() {
        let git = ScmType::Git;
        let json = serde_json::to_string(&git).unwrap();
        assert_eq!(json, "\"git\"");
        
        let deserialized: ScmType = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, ScmType::Git);
    }
}