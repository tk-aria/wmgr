use crate::domain::entities::credential::{CredentialError, CredentialProfile};
use std::process::Stdio;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tokio::time::{timeout, Duration};

const HELPER_TIMEOUT: Duration = Duration::from_secs(5);

pub struct CredentialHelperRunner;

impl CredentialHelperRunner {
    pub async fn run_helper(
        helper_command: &str,
        protocol: &str,
        host: &str,
    ) -> Result<CredentialProfile, CredentialError> {
        let input = format!("protocol={}\nhost={}\n\n", protocol, host);

        let result = timeout(HELPER_TIMEOUT, Self::execute(helper_command, &input)).await;

        match result {
            Ok(inner) => inner,
            Err(_) => Err(CredentialError::HelperFailed(format!(
                "credential helper '{}' timed out after {}s",
                helper_command,
                HELPER_TIMEOUT.as_secs()
            ))),
        }
    }

    async fn execute(
        helper_command: &str,
        input: &str,
    ) -> Result<CredentialProfile, CredentialError> {
        let parts: Vec<&str> = helper_command.split_whitespace().collect();
        if parts.is_empty() {
            return Err(CredentialError::HelperFailed(
                "empty credential helper command".to_string(),
            ));
        }

        let mut cmd = Command::new(parts[0]);
        if parts.len() > 1 {
            cmd.args(&parts[1..]);
        }
        cmd.arg("get");
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let mut child = cmd.spawn().map_err(|e| {
            CredentialError::HelperFailed(format!(
                "failed to spawn credential helper '{}': {}",
                helper_command, e
            ))
        })?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(input.as_bytes()).await.map_err(|e| {
                CredentialError::HelperFailed(format!("failed to write to helper stdin: {}", e))
            })?;
        }

        let output = child.wait_with_output().await.map_err(|e| {
            CredentialError::HelperFailed(format!("credential helper failed: {}", e))
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CredentialError::HelperFailed(format!(
                "credential helper exited with {}: {}",
                output.status, stderr
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Self::parse_helper_output(&stdout)
    }

    fn parse_helper_output(output: &str) -> Result<CredentialProfile, CredentialError> {
        let mut profile = CredentialProfile::default();

        for line in output.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if let Some((key, value)) = line.split_once('=') {
                match key.trim() {
                    "username" => profile.username = Some(value.trim().to_string()),
                    "password" => profile.password = Some(value.trim().to_string()),
                    "token" => profile.token = Some(value.trim().to_string()),
                    "aws_access_key_id" => {
                        profile.aws_access_key_id = Some(value.trim().to_string())
                    }
                    "aws_secret_access_key" => {
                        profile.aws_secret_access_key = Some(value.trim().to_string())
                    }
                    "aws_session_token" => {
                        profile.aws_session_token = Some(value.trim().to_string())
                    }
                    _ => {}
                }
            }
        }

        Ok(profile)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_helper_output() {
        let output = "username=myuser\npassword=mypass\ntoken=mytoken\n";
        let profile = CredentialHelperRunner::parse_helper_output(output).unwrap();
        assert_eq!(profile.username, Some("myuser".to_string()));
        assert_eq!(profile.password, Some("mypass".to_string()));
        assert_eq!(profile.token, Some("mytoken".to_string()));
    }

    #[test]
    fn test_parse_helper_output_with_aws() {
        let output = "aws_access_key_id=AKIA123\naws_secret_access_key=secret\n";
        let profile = CredentialHelperRunner::parse_helper_output(output).unwrap();
        assert_eq!(profile.aws_access_key_id, Some("AKIA123".to_string()));
        assert_eq!(profile.aws_secret_access_key, Some("secret".to_string()));
    }

    #[test]
    fn test_parse_helper_output_empty() {
        let profile = CredentialHelperRunner::parse_helper_output("").unwrap();
        assert!(profile.is_empty());
    }
}
