use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;
use url::Url;

/// GitURL関連のエラー
#[derive(Debug, Error, PartialEq)]
pub enum GitUrlError {
    #[error("Invalid URL format: {0}")]
    InvalidFormat(String),
    
    #[error("Unsupported URL scheme: {0}")]
    UnsupportedScheme(String),
    
    #[error("Missing host in URL")]
    MissingHost,
    
    #[error("Missing repository path")]
    MissingRepoPath,
}

/// Git URLの値オブジェクト
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GitUrl {
    /// 正規化されたURL文字列
    url: String,
    
    /// URLのスキーム（https、git、ssh等）
    scheme: String,
    
    /// ホスト名
    host: String,
    
    /// リポジトリパス（組織/リポジトリ名）
    repo_path: String,
}

impl GitUrl {
    /// 新しいGitUrlインスタンスを作成
    pub fn new(url: &str) -> Result<Self, GitUrlError> {
        let normalized = Self::normalize_url(url)?;
        let (scheme, host, repo_path) = Self::parse_url(&normalized)?;
        
        Ok(Self {
            url: normalized,
            scheme,
            host,
            repo_path,
        })
    }
    
    /// URLを正規化
    fn normalize_url(url: &str) -> Result<String, GitUrlError> {
        let trimmed = url.trim();
        
        // SSH形式（git@host:path）をhttps形式に変換
        if let Some(captures) = Regex::new(r"^git@([^:]+):(.+)$")
            .unwrap()
            .captures(trimmed) 
        {
            let host = captures.get(1).unwrap().as_str();
            let path = captures.get(2).unwrap().as_str();
            let path_without_git = path.strip_suffix(".git").unwrap_or(path);
            return Ok(format!("https://{}/{}", host, path_without_git));
        }
        
        // .gitサフィックスを除去
        let without_git_suffix = trimmed.strip_suffix(".git").unwrap_or(trimmed);
        
        // HTTPSまたはHTTPが既に含まれている場合はそのまま使用
        if without_git_suffix.starts_with("https://") || without_git_suffix.starts_with("http://") {
            return Ok(without_git_suffix.to_string());
        }
        
        // git://スキームを処理
        if without_git_suffix.starts_with("git://") {
            return Ok(without_git_suffix.replace("git://", "https://"));
        }
        
        Err(GitUrlError::InvalidFormat(url.to_string()))
    }
    
    /// URLを解析してコンポーネントに分割
    fn parse_url(url: &str) -> Result<(String, String, String), GitUrlError> {
        let parsed = Url::parse(url)
            .map_err(|_| GitUrlError::InvalidFormat(url.to_string()))?;
        
        let scheme = parsed.scheme().to_string();
        
        // サポートされるスキームのチェック
        if !matches!(scheme.as_str(), "https" | "http" | "git" | "ssh") {
            return Err(GitUrlError::UnsupportedScheme(scheme));
        }
        
        let host = parsed.host_str()
            .ok_or(GitUrlError::MissingHost)?
            .to_string();
        
        let path = parsed.path();
        if path.is_empty() || path == "/" {
            return Err(GitUrlError::MissingRepoPath);
        }
        
        // パスから先頭の/を除去
        let repo_path = path.strip_prefix('/').unwrap_or(path).to_string();
        
        Ok((scheme, host, repo_path))
    }
    
    /// 元のURL文字列を取得
    pub fn as_str(&self) -> &str {
        &self.url
    }
    
    /// スキームを取得
    pub fn scheme(&self) -> &str {
        &self.scheme
    }
    
    /// ホスト名を取得
    pub fn host(&self) -> &str {
        &self.host
    }
    
    /// リポジトリパスを取得
    pub fn repo_path(&self) -> &str {
        &self.repo_path
    }
    
    /// SSH形式のURLを生成
    pub fn to_ssh_url(&self) -> String {
        format!("git@{}:{}.git", self.host, self.repo_path)
    }
    
    /// HTTPS形式のURLを生成
    pub fn to_https_url(&self) -> String {
        format!("https://{}/{}.git", self.host, self.repo_path)
    }
    
    /// リポジトリ名を取得
    pub fn repo_name(&self) -> Option<&str> {
        self.repo_path.split('/').last()
    }
    
    /// 組織名を取得
    pub fn organization(&self) -> Option<&str> {
        let parts: Vec<&str> = self.repo_path.split('/').collect();
        if parts.len() >= 2 {
            Some(parts[0])
        } else {
            None
        }
    }
    
    /// 同じリポジトリを指しているかチェック
    pub fn is_same_repo(&self, other: &GitUrl) -> bool {
        self.host == other.host && self.repo_path == other.repo_path
    }
}

impl fmt::Display for GitUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.url)
    }
}

impl TryFrom<&str> for GitUrl {
    type Error = GitUrlError;
    
    fn try_from(url: &str) -> Result<Self, Self::Error> {
        GitUrl::new(url)
    }
}

impl TryFrom<String> for GitUrl {
    type Error = GitUrlError;
    
    fn try_from(url: String) -> Result<Self, Self::Error> {
        GitUrl::new(&url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ssh_url_normalization() {
        let git_url = GitUrl::new("git@github.com:owner/repo.git").unwrap();
        assert_eq!(git_url.as_str(), "https://github.com/owner/repo");
        assert_eq!(git_url.host(), "github.com");
        assert_eq!(git_url.repo_path(), "owner/repo");
    }
    
    #[test]
    fn test_https_url() {
        let git_url = GitUrl::new("https://github.com/owner/repo.git").unwrap();
        assert_eq!(git_url.as_str(), "https://github.com/owner/repo");
        assert_eq!(git_url.scheme(), "https");
        assert_eq!(git_url.host(), "github.com");
        assert_eq!(git_url.repo_path(), "owner/repo");
    }
    
    #[test]
    fn test_git_scheme_url() {
        let git_url = GitUrl::new("git://github.com/owner/repo").unwrap();
        assert_eq!(git_url.as_str(), "https://github.com/owner/repo");
        assert_eq!(git_url.scheme(), "https");
    }
    
    #[test]
    fn test_url_conversion() {
        let git_url = GitUrl::new("git@github.com:owner/repo.git").unwrap();
        assert_eq!(git_url.to_ssh_url(), "git@github.com:owner/repo.git");
        assert_eq!(git_url.to_https_url(), "https://github.com/owner/repo.git");
    }
    
    #[test]
    fn test_repo_info() {
        let git_url = GitUrl::new("https://github.com/owner/repo").unwrap();
        assert_eq!(git_url.repo_name(), Some("repo"));
        assert_eq!(git_url.organization(), Some("owner"));
    }
    
    #[test]
    fn test_same_repo_check() {
        let url1 = GitUrl::new("git@github.com:owner/repo.git").unwrap();
        let url2 = GitUrl::new("https://github.com/owner/repo").unwrap();
        let url3 = GitUrl::new("https://github.com/owner/other-repo").unwrap();
        
        assert!(url1.is_same_repo(&url2));
        assert!(!url1.is_same_repo(&url3));
    }
    
    #[test]
    fn test_invalid_urls() {
        assert!(GitUrl::new("invalid-url").is_err());
        assert!(GitUrl::new("ftp://example.com/repo").is_err());
        assert!(GitUrl::new("https://github.com/").is_err());
    }

    #[test]
    fn test_repo_name_extraction() {
        let git_url = GitUrl::new("https://github.com/owner/my-repo").unwrap();
        assert_eq!(git_url.repo_name(), Some("my-repo"));
        
        let git_url_deep = GitUrl::new("https://gitlab.com/group/subgroup/project").unwrap();
        assert_eq!(git_url_deep.repo_name(), Some("project"));
        
        let git_url_single = GitUrl::new("https://github.com/standalone").unwrap();
        assert_eq!(git_url_single.repo_name(), Some("standalone"));
        
        let git_url_empty = GitUrl::new("https://github.com/owner/").unwrap();
        assert_eq!(git_url_empty.repo_name(), Some(""));
    }

    #[test]
    fn test_organization_extraction() {
        let git_url = GitUrl::new("https://github.com/my-org/repo").unwrap();
        assert_eq!(git_url.organization(), Some("my-org"));
        
        let git_url_deep = GitUrl::new("https://gitlab.com/group/subgroup/project").unwrap();
        assert_eq!(git_url_deep.organization(), Some("group"));
        
        let git_url_single = GitUrl::new("https://github.com/standalone").unwrap();
        assert_eq!(git_url_single.organization(), None);
    }

    #[test]
    fn test_same_repo_comparison() {
        let ssh_url = GitUrl::new("git@github.com:owner/repo.git").unwrap();
        let https_url = GitUrl::new("https://github.com/owner/repo").unwrap();
        let https_git_url = GitUrl::new("https://github.com/owner/repo.git").unwrap();
        let different_repo = GitUrl::new("https://github.com/owner/other-repo").unwrap();
        let different_host = GitUrl::new("https://gitlab.com/owner/repo").unwrap();
        
        assert!(ssh_url.is_same_repo(&https_url));
        assert!(https_url.is_same_repo(&https_git_url));
        assert!(ssh_url.is_same_repo(&https_git_url));
        assert!(!ssh_url.is_same_repo(&different_repo));
        assert!(!https_url.is_same_repo(&different_host));
    }

    #[test]
    fn test_http_url_support() {
        let git_url = GitUrl::new("http://github.com/owner/repo").unwrap();
        assert_eq!(git_url.as_str(), "http://github.com/owner/repo");
        assert_eq!(git_url.scheme(), "http");
        assert_eq!(git_url.host(), "github.com");
        assert_eq!(git_url.repo_path(), "owner/repo");
    }

    #[test]
    fn test_edge_cases() {
        // URLにポート番号が含まれる場合
        let git_url_with_port = GitUrl::new("https://github.com:443/owner/repo").unwrap();
        assert_eq!(git_url_with_port.host(), "github.com");
        
        // パスに複数のスラッシュが含まれる場合
        let git_url_nested = GitUrl::new("https://gitlab.example.com/group/subgroup/project").unwrap();
        assert_eq!(git_url_nested.repo_path(), "group/subgroup/project");
        assert_eq!(git_url_nested.organization(), Some("group"));
        assert_eq!(git_url_nested.repo_name(), Some("project"));
    }

    #[test]
    fn test_display_trait() {
        let git_url = GitUrl::new("https://github.com/owner/repo").unwrap();
        assert_eq!(format!("{}", git_url), "https://github.com/owner/repo");
    }

    #[test]
    fn test_try_from_str() {
        let git_url: Result<GitUrl, _> = "git@github.com:owner/repo.git".try_into();
        assert!(git_url.is_ok());
        assert_eq!(git_url.unwrap().as_str(), "https://github.com/owner/repo");
    }

    #[test]
    fn test_try_from_string() {
        let url_string = String::from("https://github.com/owner/repo.git");
        let git_url: Result<GitUrl, _> = url_string.try_into();
        assert!(git_url.is_ok());
        assert_eq!(git_url.unwrap().as_str(), "https://github.com/owner/repo");
    }

    #[test]
    fn test_whitespace_handling() {
        let git_url = GitUrl::new("  https://github.com/owner/repo.git  ").unwrap();
        assert_eq!(git_url.as_str(), "https://github.com/owner/repo");
    }

    #[test]
    fn test_error_types() {
        // Invalid format
        let invalid_result = GitUrl::new("not-a-url");
        assert!(matches!(invalid_result, Err(GitUrlError::InvalidFormat(_))));
        
        // FTP URL returns InvalidFormat since it doesn't match normalization patterns
        let ftp_result = GitUrl::new("ftp://example.com/repo");
        assert!(matches!(ftp_result, Err(GitUrlError::InvalidFormat(_))));
        
        // Missing repo path
        let missing_path_result = GitUrl::new("https://github.com/");
        assert!(matches!(missing_path_result, Err(GitUrlError::MissingRepoPath)));
    }

    #[test]
    fn test_unsupported_scheme_error() {
        // To test UnsupportedScheme error, we need to create a URL that passes normalization
        // but has an unsupported scheme. We can temporarily modify the parse_url logic or
        // use a URL structure that bypasses normalization checks.
        // Since the current implementation normalizes most schemes to https, 
        // the UnsupportedScheme error is rarely reached in practice.
        // For now, we'll test that the error type exists and can be created.
        let error = GitUrlError::UnsupportedScheme("ftp".to_string());
        assert_eq!(error.to_string(), "Unsupported URL scheme: ftp");
    }
}