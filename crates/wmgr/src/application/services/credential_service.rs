use crate::domain::entities::credential::ResolvedCredentials;
use crate::domain::entities::manifest::ManifestRepo;
use crate::infrastructure::credential::CredentialStore;
use std::path::Path;

pub struct CredentialService {
    cli_profile: Option<String>,
    cli_credential_file: Option<std::path::PathBuf>,
}

impl CredentialService {
    pub fn new(
        cli_profile: Option<String>,
        cli_credential_file: Option<std::path::PathBuf>,
    ) -> Self {
        Self {
            cli_profile,
            cli_credential_file,
        }
    }

    pub async fn get_credentials_for_repo(
        &self,
        repo: &ManifestRepo,
        credential_helper: Option<&str>,
    ) -> ResolvedCredentials {
        let profile_name = CredentialStore::resolve_profile_name(
            self.cli_profile.as_deref(),
            repo.profile.as_deref(),
        );

        let credential_file_override = self.cli_credential_file.as_deref();

        let mut resolved = CredentialStore::resolve_credentials(
            &profile_name,
            credential_file_override,
            credential_helper,
        )
        .await;

        // Fallback: merge in manifest-level credentials for any fields not yet resolved
        let (manifest_username, manifest_password) = repo.get_effective_auth();
        if resolved.profile.username.is_none() {
            resolved.profile.username = manifest_username.cloned();
        }
        if resolved.profile.password.is_none() {
            resolved.profile.password = manifest_password.cloned();
        }

        resolved
    }

    pub fn cli_profile(&self) -> Option<&str> {
        self.cli_profile.as_deref()
    }

    pub fn cli_credential_file(&self) -> Option<&Path> {
        self.cli_credential_file.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_credential_service_fallback_to_manifest() {
        // Clear env vars to ensure we fall through to manifest
        std::env::remove_var("WMGR_USERNAME");
        std::env::remove_var("WMGR_PASSWORD");
        std::env::remove_var("WMGR_TOKEN");
        std::env::remove_var("WMGR_AWS_ACCESS_KEY_ID");
        std::env::remove_var("WMGR_AWS_SECRET_ACCESS_KEY");
        std::env::remove_var("WMGR_AWS_SESSION_TOKEN");
        std::env::remove_var("WMGR_GDRIVE_CLIENT_ID");
        std::env::remove_var("WMGR_GDRIVE_CLIENT_SECRET");
        std::env::remove_var("WMGR_PROFILE");

        let service = CredentialService::new(None, None);
        let repo = ManifestRepo::new("https://example.com/repo.git", "repo")
            .with_auth("manifest_user", "manifest_pass");

        let resolved = service.get_credentials_for_repo(&repo, None).await;
        assert_eq!(resolved.profile.username, Some("manifest_user".to_string()));
        assert_eq!(
            resolved.profile.password,
            Some("manifest_pass".to_string())
        );
    }
}
