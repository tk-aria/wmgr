use crate::domain::entities::credential::{
    CredentialError, CredentialFile, CredentialProfile, CredentialSource, ResolvedCredentials,
};
use std::path::{Path, PathBuf};

use super::credential_helper::CredentialHelperRunner;

pub struct CredentialStore;

impl CredentialStore {
    pub fn default_credential_file_path() -> Option<PathBuf> {
        std::env::var("HOME")
            .ok()
            .map(|home| PathBuf::from(home).join(".config/wmgr/credential.yml"))
    }

    pub async fn resolve_credentials(
        profile_name: &str,
        credential_file_override: Option<&Path>,
        credential_helper: Option<&str>,
    ) -> ResolvedCredentials {
        // Level 1: Environment variables (highest priority)
        let env_profile = Self::resolve_from_env();
        if !env_profile.is_empty() {
            return ResolvedCredentials {
                profile: env_profile,
                source: CredentialSource::EnvVar,
                profile_name: profile_name.to_string(),
            };
        }

        // Level 2: --credential-file CLI flag
        if let Some(path) = credential_file_override {
            if let Ok(profile) = Self::resolve_from_file(path, profile_name) {
                if !profile.is_empty() {
                    return ResolvedCredentials {
                        profile,
                        source: CredentialSource::CliFlag,
                        profile_name: profile_name.to_string(),
                    };
                }
            }
        }

        // Level 3: ~/.config/wmgr/credential.yml
        if let Some(default_path) = Self::default_credential_file_path() {
            if default_path.exists() {
                if let Ok(profile) = Self::resolve_from_file(&default_path, profile_name) {
                    if !profile.is_empty() {
                        return ResolvedCredentials {
                            profile,
                            source: CredentialSource::ProfileFile,
                            profile_name: profile_name.to_string(),
                        };
                    }
                }
            }
        }

        // Level 4: credential_helper external command
        if let Some(helper) = credential_helper {
            if let Ok(profile) =
                CredentialHelperRunner::run_helper(helper, "https", "default").await
            {
                if !profile.is_empty() {
                    return ResolvedCredentials {
                        profile,
                        source: CredentialSource::CredentialHelper,
                        profile_name: profile_name.to_string(),
                    };
                }
            }
        }

        ResolvedCredentials::empty(profile_name)
    }

    fn resolve_from_env() -> CredentialProfile {
        CredentialProfile {
            username: std::env::var("WMGR_USERNAME").ok(),
            password: std::env::var("WMGR_PASSWORD").ok(),
            token: std::env::var("WMGR_TOKEN").ok(),
            aws_access_key_id: std::env::var("WMGR_AWS_ACCESS_KEY_ID").ok(),
            aws_secret_access_key: std::env::var("WMGR_AWS_SECRET_ACCESS_KEY").ok(),
            aws_session_token: std::env::var("WMGR_AWS_SESSION_TOKEN").ok(),
            gdrive_client_id: std::env::var("WMGR_GDRIVE_CLIENT_ID").ok(),
            gdrive_client_secret: std::env::var("WMGR_GDRIVE_CLIENT_SECRET").ok(),
        }
    }

    fn resolve_from_file(
        path: &Path,
        profile_name: &str,
    ) -> Result<CredentialProfile, CredentialError> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = std::fs::metadata(path) {
                let mode = metadata.permissions().mode() & 0o777;
                if mode & 0o077 != 0 {
                    eprintln!(
                        "Warning: Credential file {} has insecure permissions {:o}. Consider running: chmod 600 {}",
                        path.display(),
                        mode,
                        path.display()
                    );
                }
            }
        }

        let content = std::fs::read_to_string(path).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                CredentialError::FileNotFound(path.display().to_string())
            } else if e.kind() == std::io::ErrorKind::PermissionDenied {
                CredentialError::PermissionDenied(path.display().to_string())
            } else {
                CredentialError::IoError(e.to_string())
            }
        })?;

        let credential_file: CredentialFile =
            serde_yaml::from_str(&content).map_err(|e| CredentialError::ParseError(e.to_string()))?;

        credential_file
            .get(profile_name)
            .cloned()
            .ok_or_else(|| CredentialError::ProfileNotFound(profile_name.to_string()))
    }

    pub fn resolve_profile_name(
        cli_profile: Option<&str>,
        repo_profile: Option<&str>,
    ) -> String {
        if let Some(p) = cli_profile {
            return p.to_string();
        }
        if let Some(p) = repo_profile {
            return p.to_string();
        }
        if let Ok(p) = std::env::var("WMGR_PROFILE") {
            return p;
        }
        "default".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_resolve_profile_name_priority() {
        assert_eq!(
            CredentialStore::resolve_profile_name(Some("cli"), Some("repo")),
            "cli"
        );
        assert_eq!(
            CredentialStore::resolve_profile_name(None, Some("repo")),
            "repo"
        );
        std::env::remove_var("WMGR_PROFILE");
        assert_eq!(
            CredentialStore::resolve_profile_name(None, None),
            "default"
        );
    }

    #[test]
    fn test_resolve_from_file() {
        let mut tmp = NamedTempFile::new().unwrap();
        writeln!(
            tmp,
            "default:\n  username: testuser\n  password: testpass\nwork:\n  aws_access_key_id: AKIA123"
        )
        .unwrap();

        let profile = CredentialStore::resolve_from_file(tmp.path(), "default").unwrap();
        assert_eq!(profile.username, Some("testuser".to_string()));
        assert_eq!(profile.password, Some("testpass".to_string()));

        let work_profile = CredentialStore::resolve_from_file(tmp.path(), "work").unwrap();
        assert_eq!(work_profile.aws_access_key_id, Some("AKIA123".to_string()));

        let err = CredentialStore::resolve_from_file(tmp.path(), "nonexistent");
        assert!(err.is_err());
    }

    #[test]
    fn test_resolve_from_env() {
        std::env::remove_var("WMGR_USERNAME");
        std::env::remove_var("WMGR_PASSWORD");
        let profile = CredentialStore::resolve_from_env();
        assert!(profile.username.is_none());

        std::env::set_var("WMGR_USERNAME", "envuser");
        let profile = CredentialStore::resolve_from_env();
        assert_eq!(profile.username, Some("envuser".to_string()));
        std::env::remove_var("WMGR_USERNAME");
    }
}
