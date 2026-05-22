use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CredentialSource {
    EnvVar,
    CliFlag,
    ProfileFile,
    CredentialHelper,
    Manifest,
}

impl fmt::Display for CredentialSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CredentialSource::EnvVar => write!(f, "environment variable"),
            CredentialSource::CliFlag => write!(f, "CLI flag"),
            CredentialSource::ProfileFile => write!(f, "credential profile file"),
            CredentialSource::CredentialHelper => write!(f, "credential helper"),
            CredentialSource::Manifest => write!(f, "manifest file"),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CredentialProfile {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aws_access_key_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aws_secret_access_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aws_session_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gdrive_client_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gdrive_client_secret: Option<String>,
}

impl CredentialProfile {
    pub fn is_empty(&self) -> bool {
        self.username.is_none()
            && self.password.is_none()
            && self.token.is_none()
            && self.aws_access_key_id.is_none()
            && self.aws_secret_access_key.is_none()
            && self.aws_session_token.is_none()
            && self.gdrive_client_id.is_none()
            && self.gdrive_client_secret.is_none()
    }

    pub fn merge_from(&mut self, other: &CredentialProfile) {
        if self.username.is_none() {
            self.username = other.username.clone();
        }
        if self.password.is_none() {
            self.password = other.password.clone();
        }
        if self.token.is_none() {
            self.token = other.token.clone();
        }
        if self.aws_access_key_id.is_none() {
            self.aws_access_key_id = other.aws_access_key_id.clone();
        }
        if self.aws_secret_access_key.is_none() {
            self.aws_secret_access_key = other.aws_secret_access_key.clone();
        }
        if self.aws_session_token.is_none() {
            self.aws_session_token = other.aws_session_token.clone();
        }
        if self.gdrive_client_id.is_none() {
            self.gdrive_client_id = other.gdrive_client_id.clone();
        }
        if self.gdrive_client_secret.is_none() {
            self.gdrive_client_secret = other.gdrive_client_secret.clone();
        }
    }
}

pub type CredentialFile = HashMap<String, CredentialProfile>;

#[derive(Debug, Clone)]
pub struct ResolvedCredentials {
    pub profile: CredentialProfile,
    pub source: CredentialSource,
    pub profile_name: String,
}

impl ResolvedCredentials {
    pub fn empty(profile_name: impl Into<String>) -> Self {
        Self {
            profile: CredentialProfile::default(),
            source: CredentialSource::Manifest,
            profile_name: profile_name.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CredentialError {
    FileNotFound(String),
    ParseError(String),
    ProfileNotFound(String),
    HelperFailed(String),
    PermissionDenied(String),
    IoError(String),
}

impl fmt::Display for CredentialError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CredentialError::FileNotFound(path) => {
                write!(f, "Credential file not found: {}", path)
            }
            CredentialError::ParseError(msg) => {
                write!(f, "Failed to parse credential file: {}", msg)
            }
            CredentialError::ProfileNotFound(name) => {
                write!(f, "Credential profile not found: '{}'", name)
            }
            CredentialError::HelperFailed(msg) => {
                write!(f, "Credential helper failed: {}", msg)
            }
            CredentialError::PermissionDenied(msg) => {
                write!(f, "Credential file permission denied: {}", msg)
            }
            CredentialError::IoError(msg) => {
                write!(f, "Credential I/O error: {}", msg)
            }
        }
    }
}

impl std::error::Error for CredentialError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credential_profile_is_empty() {
        assert!(CredentialProfile::default().is_empty());

        let profile = CredentialProfile {
            username: Some("user".to_string()),
            ..Default::default()
        };
        assert!(!profile.is_empty());
    }

    #[test]
    fn test_credential_profile_merge() {
        let mut base = CredentialProfile {
            username: Some("existing".to_string()),
            ..Default::default()
        };
        let other = CredentialProfile {
            username: Some("should_not_override".to_string()),
            password: Some("new_pass".to_string()),
            ..Default::default()
        };

        base.merge_from(&other);
        assert_eq!(base.username, Some("existing".to_string()));
        assert_eq!(base.password, Some("new_pass".to_string()));
    }

    #[test]
    fn test_credential_file_serde() {
        let yaml = r#"
default:
  username: "user1"
  password: "pass1"
work:
  aws_access_key_id: "AKIA..."
  aws_secret_access_key: "secret..."
"#;
        let file: CredentialFile = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(file.len(), 2);
        assert_eq!(
            file.get("default").unwrap().username,
            Some("user1".to_string())
        );
        assert_eq!(
            file.get("work").unwrap().aws_access_key_id,
            Some("AKIA...".to_string())
        );
    }

    #[test]
    fn test_credential_source_display() {
        assert_eq!(CredentialSource::EnvVar.to_string(), "environment variable");
        assert_eq!(
            CredentialSource::ProfileFile.to_string(),
            "credential profile file"
        );
    }
}
