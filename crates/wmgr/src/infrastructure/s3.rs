use aws_config::BehaviorVersion;
use aws_credential_types::Credentials;
use aws_sdk_s3::Client;
use std::path::Path;
use tokio::fs;

pub struct S3Downloader {
    client: Client,
}

#[derive(Debug, Clone, Default)]
pub struct S3Config {
    pub region: Option<String>,
    pub endpoint_url: Option<String>,
    pub access_key_id: Option<String>,
    pub secret_access_key: Option<String>,
    pub session_token: Option<String>,
}

impl S3Downloader {
    pub async fn new(config: S3Config) -> Result<Self, S3Error> {
        let mut aws_config_loader =
            aws_config::defaults(BehaviorVersion::latest());

        if let Some(region) = &config.region {
            aws_config_loader =
                aws_config_loader.region(aws_config::Region::new(region.clone()));
        }

        if let Some(endpoint) = &config.endpoint_url {
            aws_config_loader = aws_config_loader.endpoint_url(endpoint);
        }

        if let (Some(access_key), Some(secret_key)) =
            (&config.access_key_id, &config.secret_access_key)
        {
            let credentials = Credentials::new(
                access_key,
                secret_key,
                config.session_token.clone(),
                None,
                "wmgr",
            );
            aws_config_loader = aws_config_loader.credentials_provider(credentials);
        }

        let sdk_config = aws_config_loader.load().await;
        let mut s3_config_builder = aws_sdk_s3::config::Builder::from(&sdk_config);
        if config.endpoint_url.is_some() {
            s3_config_builder = s3_config_builder.force_path_style(true);
        }
        let client = Client::from_conf(s3_config_builder.build());

        Ok(Self { client })
    }

    pub async fn sync(&self, s3_url: &str, dest: &Path) -> Result<SyncResult, S3Error> {
        let (bucket, prefix) = Self::parse_s3_url(s3_url)?;

        fs::create_dir_all(dest)
            .await
            .map_err(|e| S3Error::IoError(format!("Failed to create dest dir: {}", e)))?;

        let mut continuation_token: Option<String> = None;
        let mut downloaded = 0usize;

        loop {
            let mut req = self
                .client
                .list_objects_v2()
                .bucket(&bucket)
                .prefix(&prefix);

            if let Some(token) = &continuation_token {
                req = req.continuation_token(token);
            }

            let resp = req.send().await.map_err(|e| {
                S3Error::ApiError(format!("Failed to list objects in s3://{}/{}: {}", bucket, prefix, e))
            })?;

            for obj in resp.contents() {
                if let Some(key) = obj.key() {
                    let relative = key.strip_prefix(&prefix).unwrap_or(key);
                    let relative = relative.trim_start_matches('/');

                    if relative.is_empty() || relative.ends_with('/') {
                        continue;
                    }

                    let file_path = dest.join(relative);
                    if let Some(parent) = file_path.parent() {
                        fs::create_dir_all(parent).await.map_err(|e| {
                            S3Error::IoError(format!("Failed to create dir: {}", e))
                        })?;
                    }

                    self.download_object(&bucket, key, &file_path).await?;
                    downloaded += 1;
                }
            }

            if resp.is_truncated() == Some(true) {
                continuation_token = resp.next_continuation_token().map(|s| s.to_string());
            } else {
                break;
            }
        }

        Ok(SyncResult { downloaded })
    }

    async fn download_object(
        &self,
        bucket: &str,
        key: &str,
        dest: &Path,
    ) -> Result<(), S3Error> {
        let resp = self
            .client
            .get_object()
            .bucket(bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| S3Error::ApiError(format!("Failed to get s3://{}/{}: {}", bucket, key, e)))?;

        let body = resp
            .body
            .collect()
            .await
            .map_err(|e| S3Error::ApiError(format!("Failed to read body: {}", e)))?;

        fs::write(dest, body.into_bytes())
            .await
            .map_err(|e| S3Error::IoError(format!("Failed to write {}: {}", dest.display(), e)))?;

        Ok(())
    }

    fn parse_s3_url(url: &str) -> Result<(String, String), S3Error> {
        let stripped = url
            .strip_prefix("s3://")
            .ok_or_else(|| S3Error::InvalidUrl(format!("URL must start with s3://: {}", url)))?;

        let (bucket, prefix) = match stripped.find('/') {
            Some(idx) => (
                stripped[..idx].to_string(),
                stripped[idx + 1..].to_string(),
            ),
            None => (stripped.to_string(), String::new()),
        };

        if bucket.is_empty() {
            return Err(S3Error::InvalidUrl("Bucket name is empty".to_string()));
        }

        Ok((bucket, prefix))
    }
}

pub struct SyncResult {
    pub downloaded: usize,
}

#[derive(Debug)]
pub enum S3Error {
    InvalidUrl(String),
    ApiError(String),
    IoError(String),
}

impl std::fmt::Display for S3Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            S3Error::InvalidUrl(msg) => write!(f, "Invalid S3 URL: {}", msg),
            S3Error::ApiError(msg) => write!(f, "S3 API error: {}", msg),
            S3Error::IoError(msg) => write!(f, "S3 I/O error: {}", msg),
        }
    }
}

impl std::error::Error for S3Error {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_s3_url() {
        let (bucket, prefix) = S3Downloader::parse_s3_url("s3://my-bucket/path/to/files").unwrap();
        assert_eq!(bucket, "my-bucket");
        assert_eq!(prefix, "path/to/files");

        let (bucket, prefix) = S3Downloader::parse_s3_url("s3://my-bucket").unwrap();
        assert_eq!(bucket, "my-bucket");
        assert_eq!(prefix, "");

        assert!(S3Downloader::parse_s3_url("https://example.com").is_err());
        assert!(S3Downloader::parse_s3_url("s3://").is_err());
    }
}
