use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TsrcError {
    #[error("Git operation failed: {message}")]
    GitError {
        message: String,
        #[source]
        source: Option<git2::Error>,
    },

    #[error("File system operation failed: {message}")]
    FileSystemError {
        message: String,
        path: Option<PathBuf>,
        #[source]
        source: Option<std::io::Error>,
    },

    #[error("Configuration error: {message}")]
    ConfigError {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Manifest validation error: {message}")]
    ManifestError {
        message: String,
        file_path: Option<PathBuf>,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Workspace error: {message}")]
    WorkspaceError {
        message: String,
        workspace_path: Option<PathBuf>,
    },

    #[error("Repository operation failed: {message}")]
    RepositoryError {
        message: String,
        repository_name: Option<String>,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Command execution failed: {message}")]
    CommandError {
        message: String,
        command: String,
        exit_code: Option<i32>,
        #[source]
        source: Option<std::io::Error>,
    },

    #[error("Network operation failed: {message}")]
    NetworkError {
        message: String,
        url: Option<String>,
        #[source]
        source: Option<reqwest::Error>,
    },

    #[error("Validation error: {field} - {message}")]
    ValidationError {
        field: String,
        message: String,
        value: Option<String>,
    },

    #[error("Serialization error: {message}")]
    SerializationError {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("Operation cancelled")]
    Cancelled,

    #[error("Operation timed out after {timeout_secs} seconds")]
    Timeout { timeout_secs: u64 },

    #[error("Internal error: {message}")]
    InternalError {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

impl TsrcError {
    pub fn git_error(message: impl Into<String>) -> Self {
        Self::GitError {
            message: message.into(),
            source: None,
        }
    }

    pub fn git_error_with_source(message: impl Into<String>, source: git2::Error) -> Self {
        Self::GitError {
            message: message.into(),
            source: Some(source),
        }
    }

    pub fn filesystem_error(message: impl Into<String>, path: Option<PathBuf>) -> Self {
        Self::FileSystemError {
            message: message.into(),
            path,
            source: None,
        }
    }

    pub fn filesystem_error_with_source(
        message: impl Into<String>,
        path: Option<PathBuf>,
        source: std::io::Error,
    ) -> Self {
        Self::FileSystemError {
            message: message.into(),
            path,
            source: Some(source),
        }
    }

    pub fn config_error(message: impl Into<String>) -> Self {
        Self::ConfigError {
            message: message.into(),
            source: None,
        }
    }

    pub fn config_error_with_source(
        message: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self::ConfigError {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    pub fn manifest_error(message: impl Into<String>, file_path: Option<PathBuf>) -> Self {
        Self::ManifestError {
            message: message.into(),
            file_path,
            source: None,
        }
    }

    pub fn manifest_error_with_source(
        message: impl Into<String>,
        file_path: Option<PathBuf>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self::ManifestError {
            message: message.into(),
            file_path,
            source: Some(Box::new(source)),
        }
    }

    pub fn workspace_error(message: impl Into<String>, workspace_path: Option<PathBuf>) -> Self {
        Self::WorkspaceError {
            message: message.into(),
            workspace_path,
        }
    }

    pub fn repository_error(message: impl Into<String>, repository_name: Option<String>) -> Self {
        Self::RepositoryError {
            message: message.into(),
            repository_name,
            source: None,
        }
    }

    pub fn repository_error_with_source(
        message: impl Into<String>,
        repository_name: Option<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self::RepositoryError {
            message: message.into(),
            repository_name,
            source: Some(Box::new(source)),
        }
    }

    pub fn command_error(
        message: impl Into<String>,
        command: impl Into<String>,
        exit_code: Option<i32>,
    ) -> Self {
        Self::CommandError {
            message: message.into(),
            command: command.into(),
            exit_code,
            source: None,
        }
    }

    pub fn command_error_with_source(
        message: impl Into<String>,
        command: impl Into<String>,
        exit_code: Option<i32>,
        source: std::io::Error,
    ) -> Self {
        Self::CommandError {
            message: message.into(),
            command: command.into(),
            exit_code,
            source: Some(source),
        }
    }

    pub fn network_error(message: impl Into<String>, url: Option<String>) -> Self {
        Self::NetworkError {
            message: message.into(),
            url,
            source: None,
        }
    }

    pub fn network_error_with_source(
        message: impl Into<String>,
        url: Option<String>,
        source: reqwest::Error,
    ) -> Self {
        Self::NetworkError {
            message: message.into(),
            url,
            source: Some(source),
        }
    }

    pub fn validation_error(
        field: impl Into<String>,
        message: impl Into<String>,
        value: Option<String>,
    ) -> Self {
        Self::ValidationError {
            field: field.into(),
            message: message.into(),
            value,
        }
    }

    pub fn serialization_error(message: impl Into<String>) -> Self {
        Self::SerializationError {
            message: message.into(),
            source: None,
        }
    }

    pub fn serialization_error_with_source(
        message: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self::SerializationError {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }

    pub fn timeout(timeout_secs: u64) -> Self {
        Self::Timeout { timeout_secs }
    }

    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::InternalError {
            message: message.into(),
            source: None,
        }
    }

    pub fn internal_error_with_source(
        message: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self::InternalError {
            message: message.into(),
            source: Some(Box::new(source)),
        }
    }
}

impl From<git2::Error> for TsrcError {
    fn from(error: git2::Error) -> Self {
        Self::git_error_with_source("Git operation failed", error)
    }
}

impl From<std::io::Error> for TsrcError {
    fn from(error: std::io::Error) -> Self {
        Self::filesystem_error_with_source("File system operation failed", None, error)
    }
}

impl From<serde_yaml::Error> for TsrcError {
    fn from(error: serde_yaml::Error) -> Self {
        Self::serialization_error_with_source("YAML serialization failed", error)
    }
}

impl From<serde_json::Error> for TsrcError {
    fn from(error: serde_json::Error) -> Self {
        Self::serialization_error_with_source("JSON serialization failed", error)
    }
}

impl From<reqwest::Error> for TsrcError {
    fn from(error: reqwest::Error) -> Self {
        Self::network_error_with_source("Network request failed", None, error)
    }
}

impl From<anyhow::Error> for TsrcError {
    fn from(error: anyhow::Error) -> Self {
        Self::internal_error(format!("Anyhow error: {}", error))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_git_error_creation() {
        let error = TsrcError::git_error("test message");
        assert!(matches!(error, TsrcError::GitError { .. }));
        assert_eq!(error.to_string(), "Git operation failed: test message");
    }

    #[test]
    fn test_filesystem_error_with_path() {
        let path = PathBuf::from("/test/path");
        let error = TsrcError::filesystem_error("test message", Some(path.clone()));
        if let TsrcError::FileSystemError { path: Some(p), .. } = error {
            assert_eq!(p, path);
        } else {
            panic!("Expected FileSystemError with path");
        }
    }

    #[test]
    fn test_validation_error() {
        let error = TsrcError::validation_error("field", "message", Some("value".to_string()));
        assert_eq!(error.to_string(), "Validation error: field - message");
    }

    #[test]
    fn test_timeout_error() {
        let error = TsrcError::timeout(30);
        assert_eq!(error.to_string(), "Operation timed out after 30 seconds");
    }

    #[test]
    fn test_error_conversion_from_io_error() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let tsrc_error: TsrcError = io_error.into();
        assert!(matches!(tsrc_error, TsrcError::FileSystemError { .. }));
    }
}
