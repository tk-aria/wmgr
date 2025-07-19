use crate::domain::entities::manifest::{Group, Manifest, ManifestRepo};
use crate::domain::value_objects::git_url::GitUrl;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// ManifestService関連のエラー
#[derive(Debug, Error)]
pub enum ManifestServiceError {
    #[error("Failed to parse manifest: {0}")]
    ParseError(String),

    #[error("Manifest validation failed: {0}")]
    ValidationError(String),

    #[error("Invalid YAML format: {0}")]
    YamlError(#[from] serde_yaml::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("Invalid Git URL: {0}")]
    GitUrlError(#[from] crate::domain::value_objects::git_url::GitUrlError),

    #[error("Group '{0}' not found in manifest")]
    GroupNotFound(String),

    #[error("Circular dependency detected in manifest chain: {0}")]
    CircularDependency(String),

    #[error("Deep manifest depth limit exceeded (max: {max}, current: {current})")]
    DepthLimitExceeded { max: usize, current: usize },

    #[error("Failed to fetch remote manifest from {url}: {reason}")]
    RemoteManifestFetchFailed { url: String, reason: String },

    #[error("JSON serialization error: {0}")]
    JsonError(#[from] serde_json::Error),
}

/// DeepManifest/FutureManifestのサポート設定
#[derive(Debug, Clone)]
pub struct ManifestProcessingOptions {
    /// Deep manifestの最大深度
    pub max_depth: usize,

    /// リモートマニフェストの取得を有効にするか
    pub enable_remote_fetch: bool,

    /// 循環依存の検出を有効にするか
    pub detect_circular_dependencies: bool,

    /// タイムアウト設定（秒）
    pub timeout_seconds: u64,
}

impl Default for ManifestProcessingOptions {
    fn default() -> Self {
        Self {
            max_depth: 10,
            enable_remote_fetch: true,
            detect_circular_dependencies: true,
            timeout_seconds: 30,
        }
    }
}

/// 拡張マニフェスト定義（Deep/Future Manifest対応）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtendedManifest {
    /// 基本のマニフェスト情報
    #[serde(flatten)]
    pub manifest: Manifest,

    /// インクルードする他のマニフェスト
    #[serde(skip_serializing_if = "Option::is_none")]
    pub includes: Option<Vec<ManifestInclude>>,

    /// 将来のバージョンでのマニフェスト設定
    #[serde(skip_serializing_if = "Option::is_none")]
    pub future: Option<FutureManifestConfig>,
}

/// マニフェストインクルード定義
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestInclude {
    /// インクルードするマニフェストのURL
    pub url: String,

    /// インクルードするマニフェストのブランチ/リビジョン
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revision: Option<String>,

    /// インクルードしたマニフェストから特定のグループのみを使用
    #[serde(skip_serializing_if = "Option::is_none")]
    pub groups: Option<Vec<String>>,

    /// インクルードの優先度（高い方が優先）
    #[serde(default)]
    pub priority: i32,
}

/// Future Manifest設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FutureManifestConfig {
    /// 最小バージョン要件
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_version: Option<String>,

    /// 廃止予定の機能
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deprecated: Option<Vec<DeprecationWarning>>,

    /// 将来のデフォルト設定
    #[serde(skip_serializing_if = "Option::is_none")]
    pub future_defaults: Option<HashMap<String, serde_yaml::Value>>,
}

/// 廃止予定警告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeprecationWarning {
    /// 廃止予定の機能名
    pub feature: String,

    /// 警告メッセージ
    pub message: String,

    /// 廃止予定バージョン
    #[serde(skip_serializing_if = "Option::is_none")]
    pub removal_version: Option<String>,
}

/// マニフェスト処理結果
#[derive(Debug, Clone)]
pub struct ProcessedManifest {
    /// 処理されたマニフェスト
    pub manifest: Manifest,

    /// 処理中に発生した警告
    pub warnings: Vec<String>,

    /// インクルードされたマニフェストの情報
    pub includes: Vec<IncludeInfo>,
}

/// インクルード情報
#[derive(Debug, Clone)]
pub struct IncludeInfo {
    /// インクルード元のURL
    pub url: String,

    /// 実際に使用されたリビジョン
    pub revision: String,

    /// インクルードから取得されたリポジトリ数
    pub repo_count: usize,
}

/// マニフェストサービス
pub struct ManifestService {
    /// 処理オプション
    options: ManifestProcessingOptions,

    /// HTTPクライアント
    http_client: reqwest::Client,

    /// 処理済みマニフェストのキャッシュ
    cache: HashMap<String, ExtendedManifest>,
}

impl ManifestService {
    /// 新しいManifestServiceインスタンスを作成
    pub fn new(options: ManifestProcessingOptions) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(options.timeout_seconds))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            options,
            http_client,
            cache: HashMap::new(),
        }
    }

    /// デフォルト設定でManifestServiceを作成
    pub fn default() -> Self {
        Self::new(ManifestProcessingOptions::default())
    }

    /// ローカルファイルからマニフェストを読み込み・解析
    pub async fn parse_from_file(
        &mut self,
        path: &Path,
    ) -> Result<ProcessedManifest, ManifestServiceError> {
        let content = tokio::fs::read_to_string(path).await?;
        self.parse_from_string(&content, Some(path.to_path_buf()))
            .await
    }

    /// URL からマニフェストを読み込み・解析
    pub async fn parse_from_url(
        &mut self,
        url: &str,
    ) -> Result<ProcessedManifest, ManifestServiceError> {
        // キャッシュをチェック
        if let Some(cached) = self.cache.get(url) {
            return Ok(ProcessedManifest {
                manifest: cached.manifest.clone(),
                warnings: vec![],
                includes: vec![],
            });
        }

        let content = self.fetch_remote_content(url).await?;
        let result = self.parse_from_string(&content, None).await?;

        Ok(result)
    }

    /// 文字列からマニフェストを解析
    pub async fn parse_from_string(
        &mut self,
        content: &str,
        base_path: Option<PathBuf>,
    ) -> Result<ProcessedManifest, ManifestServiceError> {
        // 基本のYAMLパース
        let extended_manifest: ExtendedManifest = serde_yaml::from_str(content)?;

        // バリデーション
        self.validate_manifest(&extended_manifest.manifest)?;

        // Deep manifest処理
        let processed = self
            .process_deep_manifest(extended_manifest, base_path, 0, &mut Vec::new())
            .await?;

        Ok(processed)
    }

    /// マニフェストを検証
    pub fn validate_manifest(&self, manifest: &Manifest) -> Result<(), ManifestServiceError> {
        // 重複するdestの検証
        let mut dest_set = std::collections::HashSet::new();
        for repo in &manifest.repos {
            if !dest_set.insert(&repo.dest) {
                return Err(ManifestServiceError::ValidationError(format!(
                    "Duplicate destination path: {}",
                    repo.dest
                )));
            }

            // URL検証
            GitUrl::new(&repo.url)?;
        }

        // グループ検証
        if let Some(groups) = &manifest.groups {
            for (group_name, group) in groups {
                for repo_dest in &group.repos {
                    if !manifest.repos.iter().any(|r| &r.dest == repo_dest) {
                        return Err(ManifestServiceError::ValidationError(format!(
                            "Group '{}' references non-existent repository: {}",
                            group_name, repo_dest
                        )));
                    }
                }
            }
        }

        Ok(())
    }

    /// 指定されたグループでマニフェストをフィルタリング
    pub fn filter_by_groups(
        &self,
        manifest: &Manifest,
        group_names: &[String],
    ) -> Result<Manifest, ManifestServiceError> {
        if group_names.is_empty() {
            return Ok(manifest.clone());
        }

        let mut filtered_repos = Vec::new();
        let mut repo_dests = std::collections::HashSet::new();

        // 各グループからリポジトリを収集
        for group_name in group_names {
            let group_repos = manifest.get_repos_in_group(group_name);
            if group_repos.is_empty() {
                return Err(ManifestServiceError::GroupNotFound(group_name.clone()));
            }

            for repo in group_repos {
                if repo_dests.insert(&repo.dest) {
                    filtered_repos.push(repo.clone());
                }
            }
        }

        // フィルタされたグループ定義も作成
        let mut filtered_groups = HashMap::new();
        if let Some(groups) = &manifest.groups {
            for group_name in group_names {
                if let Some(group) = groups.get(group_name) {
                    // グループ内のリポジトリをフィルタされたもののみに限定
                    let filtered_group_repos: Vec<String> = group
                        .repos
                        .iter()
                        .filter(|dest| repo_dests.contains(dest))
                        .cloned()
                        .collect();

                    if !filtered_group_repos.is_empty() {
                        let mut filtered_group = group.clone();
                        filtered_group.repos = filtered_group_repos;
                        filtered_groups.insert(group_name.clone(), filtered_group);
                    }
                }
            }
        }

        let mut filtered_manifest = Manifest::new(filtered_repos);
        filtered_manifest.default_branch = manifest.default_branch.clone();

        if !filtered_groups.is_empty() {
            filtered_manifest = filtered_manifest.with_groups(filtered_groups);
        }

        Ok(filtered_manifest)
    }

    /// グループ一覧を取得
    pub fn list_groups(&self, manifest: &Manifest) -> Vec<String> {
        if let Some(groups) = &manifest.groups {
            groups.keys().cloned().collect()
        } else {
            Vec::new()
        }
    }

    /// 特定のグループの詳細情報を取得
    pub fn get_group_info<'a>(
        &self,
        manifest: &'a Manifest,
        group_name: &str,
    ) -> Option<(&'a Group, Vec<&'a ManifestRepo>)> {
        if let Some(groups) = &manifest.groups {
            if let Some(group) = groups.get(group_name) {
                let repos = manifest.get_repos_in_group(group_name);
                return Some((group, repos));
            }
        }
        None
    }

    /// Deep manifestを処理
    fn process_deep_manifest<'a>(
        &'a mut self,
        mut extended_manifest: ExtendedManifest,
        base_path: Option<PathBuf>,
        depth: usize,
        visited: &'a mut Vec<String>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<ProcessedManifest, ManifestServiceError>> + 'a>,
    > {
        Box::pin(async move {
            // 深度制限チェック
            if depth > self.options.max_depth {
                return Err(ManifestServiceError::DepthLimitExceeded {
                    max: self.options.max_depth,
                    current: depth,
                });
            }

            let mut warnings = Vec::new();
            let mut all_includes = Vec::new();

            // Future manifest設定の処理
            if let Some(future_config) = &extended_manifest.future {
                warnings.extend(self.process_future_config(future_config));
            }

            // インクルードの処理
            if let Some(includes) = &extended_manifest.includes {
                for include in includes {
                    let include_url = self.resolve_include_url(&include.url, &base_path)?;

                    // 循環依存チェック
                    if self.options.detect_circular_dependencies && visited.contains(&include_url) {
                        return Err(ManifestServiceError::CircularDependency(format!(
                            "Circular dependency detected: {} -> {}",
                            visited.join(" -> "),
                            include_url
                        )));
                    }

                    visited.push(include_url.clone());

                    // インクルードマニフェストを取得・処理
                    let included_content = self.fetch_remote_content(&include_url).await?;
                    let included_extended: ExtendedManifest =
                        serde_yaml::from_str(&included_content)?;

                    let included_processed = self
                        .process_deep_manifest(included_extended, None, depth + 1, visited)
                        .await?;

                    visited.pop();

                    // インクルードされたマニフェストをマージ
                    let filtered_manifest = if let Some(groups) = &include.groups {
                        self.filter_by_groups(&included_processed.manifest, groups)?
                    } else {
                        included_processed.manifest
                    };

                    // インクルード情報を記録（マージ前に）
                    let repo_count = filtered_manifest.repos.len();

                    extended_manifest.manifest = self.merge_manifests(
                        extended_manifest.manifest,
                        filtered_manifest,
                        include.priority,
                    )?;
                    all_includes.push(IncludeInfo {
                        url: include_url,
                        revision: include
                            .revision
                            .clone()
                            .unwrap_or_else(|| "HEAD".to_string()),
                        repo_count,
                    });

                    warnings.extend(included_processed.warnings);
                    all_includes.extend(included_processed.includes);
                }
            }

            Ok(ProcessedManifest {
                manifest: extended_manifest.manifest,
                warnings,
                includes: all_includes,
            })
        })
    }

    /// Future manifest設定を処理
    fn process_future_config(&self, future_config: &FutureManifestConfig) -> Vec<String> {
        let mut warnings = Vec::new();

        // バージョンチェック（簡易実装）
        if let Some(min_version) = &future_config.min_version {
            warnings.push(format!("Minimum version requirement: {}", min_version));
        }

        // 廃止予定警告
        if let Some(deprecated) = &future_config.deprecated {
            for dep in deprecated {
                let warning = if let Some(removal_version) = &dep.removal_version {
                    format!(
                        "DEPRECATED: {} (will be removed in {}): {}",
                        dep.feature, removal_version, dep.message
                    )
                } else {
                    format!("DEPRECATED: {}: {}", dep.feature, dep.message)
                };
                warnings.push(warning);
            }
        }

        warnings
    }

    /// インクルードURLを解決
    fn resolve_include_url(
        &self,
        url: &str,
        base_path: &Option<PathBuf>,
    ) -> Result<String, ManifestServiceError> {
        // 既に絶対URLの場合はそのまま返す
        if url.starts_with("http://") || url.starts_with("https://") || url.starts_with("git@") {
            return Ok(url.to_string());
        }

        // 相対パスの場合、base_pathからの相対パスとして解決
        if let Some(base) = base_path {
            if let Some(parent) = base.parent() {
                let resolved = parent.join(url);
                return Ok(resolved.to_string_lossy().to_string());
            }
        }

        Ok(url.to_string())
    }

    /// リモートコンテンツを取得
    async fn fetch_remote_content(&self, url: &str) -> Result<String, ManifestServiceError> {
        if !self.options.enable_remote_fetch {
            return Err(ManifestServiceError::RemoteManifestFetchFailed {
                url: url.to_string(),
                reason: "Remote fetch is disabled".to_string(),
            });
        }

        if url.starts_with("http://") || url.starts_with("https://") {
            let response = self.http_client.get(url).send().await?;

            if response.status().is_success() {
                Ok(response.text().await?)
            } else {
                Err(ManifestServiceError::RemoteManifestFetchFailed {
                    url: url.to_string(),
                    reason: format!("HTTP {}", response.status()),
                })
            }
        } else {
            // ローカルファイルとして扱う
            match tokio::fs::read_to_string(url).await {
                Ok(content) => Ok(content),
                Err(e) => Err(ManifestServiceError::RemoteManifestFetchFailed {
                    url: url.to_string(),
                    reason: e.to_string(),
                }),
            }
        }
    }

    /// マニフェストをマージ
    fn merge_manifests(
        &self,
        mut base: Manifest,
        include: Manifest,
        _priority: i32,
    ) -> Result<Manifest, ManifestServiceError> {
        // リポジトリをマージ（重複するdestは基本的にbaseを優先）
        let mut dest_set: std::collections::HashSet<String> =
            base.repos.iter().map(|r| r.dest.clone()).collect();

        for repo in include.repos {
            if !dest_set.contains(&repo.dest) {
                dest_set.insert(repo.dest.clone());
                base.repos.push(repo);
            }
        }

        // グループをマージ
        if let Some(include_groups) = include.groups {
            let mut merged_groups = base.groups.unwrap_or_default();

            for (group_name, group) in include_groups {
                // 既存のグループがある場合はリポジトリをマージ
                if let Some(existing_group) = merged_groups.get_mut(&group_name) {
                    for repo_dest in group.repos {
                        if !existing_group.repos.contains(&repo_dest) {
                            existing_group.repos.push(repo_dest);
                        }
                    }
                } else {
                    merged_groups.insert(group_name, group);
                }
            }

            base.groups = Some(merged_groups);
        }

        // デフォルトブランチはbaseを優先
        if base.default_branch.is_none() {
            base.default_branch = include.default_branch;
        }

        Ok(base)
    }

    /// マニフェストをYAML文字列にシリアライズ
    pub fn serialize_to_yaml(&self, manifest: &Manifest) -> Result<String, ManifestServiceError> {
        Ok(serde_yaml::to_string(manifest)?)
    }

    /// マニフェストをJSON文字列にシリアライズ
    pub fn serialize_to_json(&self, manifest: &Manifest) -> Result<String, ManifestServiceError> {
        Ok(serde_json::to_string_pretty(manifest)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::manifest::{Group, ManifestRepo};
    use std::collections::HashMap;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_parse_basic_manifest() {
        let yaml_content = r#"
repos:
  - dest: repo1
    url: https://github.com/example/repo1.git
    branch: main
  - dest: repo2
    url: https://github.com/example/repo2.git
    branch: develop

groups:
  group1:
    repos:
      - repo1
      - repo2
    description: "Test group"

default_branch: main
"#;

        let mut service = ManifestService::default();
        let result = service.parse_from_string(yaml_content, None).await.unwrap();

        assert_eq!(result.manifest.repos.len(), 2);
        assert_eq!(result.manifest.repos[0].dest, "repo1");
        assert_eq!(result.manifest.repos[1].dest, "repo2");

        let groups = result.manifest.groups.unwrap();
        assert!(groups.contains_key("group1"));
        assert_eq!(groups["group1"].repos.len(), 2);

        assert_eq!(result.manifest.default_branch, Some("main".to_string()));
    }

    #[tokio::test]
    async fn test_manifest_validation() {
        let yaml_content = r#"
repos:
  - dest: repo1
    url: https://github.com/example/repo1.git
  - dest: repo1  # 重複するdest
    url: https://github.com/example/repo2.git
"#;

        let mut service = ManifestService::default();
        let result = service.parse_from_string(yaml_content, None).await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ManifestServiceError::ValidationError(_)
        ));
    }

    #[tokio::test]
    async fn test_filter_by_groups() {
        let repos = vec![
            ManifestRepo::new("https://github.com/example/repo1.git", "repo1"),
            ManifestRepo::new("https://github.com/example/repo2.git", "repo2"),
            ManifestRepo::new("https://github.com/example/repo3.git", "repo3"),
        ];

        let mut groups = HashMap::new();
        groups.insert(
            "group1".to_string(),
            Group::new(vec!["repo1".to_string(), "repo2".to_string()]),
        );
        groups.insert("group2".to_string(), Group::new(vec!["repo3".to_string()]));

        let manifest = Manifest::new(repos).with_groups(groups);
        let service = ManifestService::default();

        let filtered = service
            .filter_by_groups(&manifest, &["group1".to_string()])
            .unwrap();

        assert_eq!(filtered.repos.len(), 2);
        assert!(filtered.repos.iter().any(|r| r.dest == "repo1"));
        assert!(filtered.repos.iter().any(|r| r.dest == "repo2"));
        assert!(!filtered.repos.iter().any(|r| r.dest == "repo3"));

        let groups = filtered.groups.unwrap();
        assert!(groups.contains_key("group1"));
        assert!(!groups.contains_key("group2"));
    }

    #[tokio::test]
    async fn test_group_not_found() {
        let manifest = Manifest::new(vec![]);
        let service = ManifestService::default();

        let result = service.filter_by_groups(&manifest, &["nonexistent".to_string()]);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ManifestServiceError::GroupNotFound(_)
        ));
    }

    #[test]
    fn test_list_groups() {
        let mut groups = HashMap::new();
        groups.insert("group1".to_string(), Group::new(vec![]));
        groups.insert("group2".to_string(), Group::new(vec![]));

        let manifest = Manifest::new(vec![]).with_groups(groups);
        let service = ManifestService::default();

        let mut group_list = service.list_groups(&manifest);
        group_list.sort();

        assert_eq!(group_list, vec!["group1", "group2"]);
    }

    #[test]
    fn test_get_group_info() {
        let repos = vec![
            ManifestRepo::new("https://github.com/example/repo1.git", "repo1"),
            ManifestRepo::new("https://github.com/example/repo2.git", "repo2"),
        ];

        let mut groups = HashMap::new();
        groups.insert(
            "group1".to_string(),
            Group::new(vec!["repo1".to_string(), "repo2".to_string()])
                .with_description("Test group"),
        );

        let manifest = Manifest::new(repos).with_groups(groups);
        let service = ManifestService::default();

        let (group, group_repos) = service.get_group_info(&manifest, "group1").unwrap();
        assert_eq!(group.description, Some("Test group".to_string()));
        assert_eq!(group_repos.len(), 2);

        assert!(service.get_group_info(&manifest, "nonexistent").is_none());
    }

    #[tokio::test]
    async fn test_parse_from_file() {
        let temp_dir = TempDir::new().unwrap();
        let manifest_file = temp_dir.path().join("manifest.yml");

        let yaml_content = r#"
repos:
  - dest: test-repo
    url: https://github.com/example/test-repo.git
    branch: main
"#;

        tokio::fs::write(&manifest_file, yaml_content)
            .await
            .unwrap();

        let mut service = ManifestService::default();
        let result = service.parse_from_file(&manifest_file).await.unwrap();

        assert_eq!(result.manifest.repos.len(), 1);
        assert_eq!(result.manifest.repos[0].dest, "test-repo");
    }

    #[tokio::test]
    async fn test_extended_manifest_with_includes() {
        let yaml_content = r#"
repos:
  - dest: main-repo
    url: https://github.com/example/main-repo.git

includes:
  - url: https://example.com/other-manifest.yml
    priority: 1
    groups:
      - core

future:
  min_version: "1.0.0"
  deprecated:
    - feature: "old_feature"
      message: "Use new_feature instead"
      removal_version: "2.0.0"
"#;

        // リモートフェッチを無効にしてテスト
        let mut options = ManifestProcessingOptions::default();
        options.enable_remote_fetch = false;

        let mut service = ManifestService::new(options);
        let result = service.parse_from_string(yaml_content, None).await;

        // リモートフェッチが無効なのでエラーになるはず
        assert!(result.is_err());
    }

    #[test]
    fn test_serialize_manifest() {
        let repos = vec![ManifestRepo::new(
            "https://github.com/example/repo1.git",
            "repo1",
        )];

        let manifest = Manifest::new(repos);
        let service = ManifestService::default();

        let yaml_output = service.serialize_to_yaml(&manifest).unwrap();
        assert!(yaml_output.contains("repos:"));
        assert!(yaml_output.contains("dest: repo1"));

        let json_output = service.serialize_to_json(&manifest).unwrap();
        assert!(json_output.contains("\"repos\""));
        assert!(json_output.contains("\"dest\": \"repo1\""));
    }

    #[test]
    fn test_depth_limit() {
        let mut options = ManifestProcessingOptions::default();
        options.max_depth = 2;

        let service = ManifestService::new(options);

        assert_eq!(service.options.max_depth, 2);
    }

    #[test]
    fn test_merge_manifests() {
        let service = ManifestService::default();

        let base_repos = vec![ManifestRepo::new(
            "https://github.com/example/repo1.git",
            "repo1",
        )];
        let base_manifest = Manifest::new(base_repos);

        let include_repos = vec![
            ManifestRepo::new("https://github.com/example/repo2.git", "repo2"),
            ManifestRepo::new("https://github.com/example/repo1.git", "repo1"), // 重複
        ];
        let include_manifest = Manifest::new(include_repos);

        let merged = service
            .merge_manifests(base_manifest, include_manifest, 1)
            .unwrap();

        // 重複するdestは無視されるため、2つのリポジトリになる
        assert_eq!(merged.repos.len(), 2);
        assert!(merged.repos.iter().any(|r| r.dest == "repo1"));
        assert!(merged.repos.iter().any(|r| r.dest == "repo2"));
    }
}
