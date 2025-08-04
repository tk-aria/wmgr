use super::manifest::{ManifestRepo, ScmOptions};
use crate::domain::value_objects::scm_type::ScmType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// ワークスペース設定のデフォルト値
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Defaults {
    /// デフォルトのSCMタイプ
    #[serde(default)]
    pub scm: ScmType,

    /// デフォルトのブランチ名（Git用）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,

    /// デフォルトのリモート名（Git用）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote: Option<String>,

    /// デフォルトで浅いクローンを使用するか（Git用）
    #[serde(default)]
    pub shallow: bool,
}

impl Default for Defaults {
    fn default() -> Self {
        Self {
            scm: ScmType::Git,
            branch: None,
            remote: None,
            shallow: false,
        }
    }
}

/// ワークスペース情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceInfo {
    /// ワークスペース名
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// ワークスペースの説明
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// リポジトリグループ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryGroup {
    /// グループ名
    pub name: String,

    /// グループの説明
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// グループに含まれるリポジトリ
    pub repositories: Vec<ManifestRepo>,
}

impl RepositoryGroup {
    /// 新しいグループを作成
    pub fn new(name: impl Into<String>, repositories: Vec<ManifestRepo>) -> Self {
        Self {
            name: name.into(),
            description: None,
            repositories,
        }
    }

    /// 説明を設定
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

/// wmgr.yaml設定ファイルの構造
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    /// デフォルト設定
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defaults: Option<Defaults>,

    /// ワークスペース情報
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace: Option<WorkspaceInfo>,

    /// リポジトリグループ（新しい推奨形式）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub groups: Option<Vec<RepositoryGroup>>,

    /// フラットなリポジトリリスト（旧形式との互換性）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repositories: Option<Vec<ManifestRepo>>,
}

impl WorkspaceConfig {
    /// 空の設定を作成
    pub fn new() -> Self {
        Self {
            defaults: None,
            workspace: None,
            groups: None,
            repositories: None,
        }
    }

    /// デフォルト設定を追加
    pub fn with_defaults(mut self, defaults: Defaults) -> Self {
        self.defaults = Some(defaults);
        self
    }

    /// ワークスペース情報を追加
    pub fn with_workspace_info(mut self, workspace: WorkspaceInfo) -> Self {
        self.workspace = Some(workspace);
        self
    }

    /// グループを追加
    pub fn with_groups(mut self, groups: Vec<RepositoryGroup>) -> Self {
        self.groups = Some(groups);
        self
    }

    /// フラットなリポジトリリストを追加
    pub fn with_repositories(mut self, repositories: Vec<ManifestRepo>) -> Self {
        self.repositories = Some(repositories);
        self
    }

    /// デフォルト設定を適用してリポジトリリストを正規化
    pub fn normalize_repositories(&self) -> Vec<ManifestRepo> {
        let defaults = self.defaults.as_ref();
        let mut all_repos = Vec::new();

        // グループからリポジトリを収集
        if let Some(groups) = &self.groups {
            for group in groups {
                for repo in &group.repositories {
                    all_repos.push(self.apply_defaults_to_repo(repo, defaults));
                }
            }
        }

        // フラットなリポジトリリストを追加
        if let Some(repositories) = &self.repositories {
            for repo in repositories {
                all_repos.push(self.apply_defaults_to_repo(repo, defaults));
            }
        }

        all_repos
    }

    /// デフォルト設定をリポジトリに適用
    fn apply_defaults_to_repo(&self, repo: &ManifestRepo, defaults: Option<&Defaults>) -> ManifestRepo {
        let mut normalized = repo.clone();

        if let Some(defaults) = defaults {
            // SCMがデフォルトの場合、デフォルト設定を適用
            if normalized.scm == ScmType::default() {
                normalized.scm = defaults.scm.clone();
            }

            // ブランチがNoneでGitの場合、デフォルトブランチを適用
            if normalized.branch.is_none() && normalized.scm == ScmType::Git {
                if let Some(default_branch) = &defaults.branch {
                    normalized.branch = Some(default_branch.clone());
                }
            }

            // 浅いクローンの設定を適用（Git用）
            if normalized.scm == ScmType::Git && !normalized.shallow && defaults.shallow {
                normalized.shallow = defaults.shallow;
            }
        }

        normalized
    }

    /// 特定のグループのリポジトリを取得
    pub fn get_repositories_in_group(&self, group_name: &str) -> Vec<ManifestRepo> {
        if let Some(groups) = &self.groups {
            for group in groups {
                if group.name == group_name {
                    return group.repositories.iter()
                        .map(|repo| self.apply_defaults_to_repo(repo, self.defaults.as_ref()))
                        .collect();
                }
            }
        }
        Vec::new()
    }

    /// 全てのグループ名を取得
    pub fn get_group_names(&self) -> Vec<String> {
        if let Some(groups) = &self.groups {
            groups.iter().map(|g| g.name.clone()).collect()
        } else {
            Vec::new()
        }
    }

    /// destパスでリポジトリを検索
    pub fn find_repository_by_dest(&self, dest: &str) -> Option<ManifestRepo> {
        let all_repos = self.normalize_repositories();
        all_repos.into_iter().find(|repo| repo.dest == dest)
    }

    /// 設定の妥当性をチェック
    pub fn validate(&self) -> Result<(), String> {
        // グループとフラットリポジトリの両方が空でないことをチェック
        if self.groups.is_none() && self.repositories.is_none() {
            return Err("Either 'groups' or 'repositories' must be specified".to_string());
        }

        // グループ内のリポジトリの妥当性チェック
        if let Some(groups) = &self.groups {
            for group in groups {
                if group.repositories.is_empty() {
                    return Err(format!("Group '{}' has no repositories", group.name));
                }
                
                for repo in &group.repositories {
                    self.validate_repository(repo)?;
                }
            }
        }

        // フラットリポジトリの妥当性チェック
        if let Some(repositories) = &self.repositories {
            for repo in repositories {
                self.validate_repository(repo)?;
            }
        }

        Ok(())
    }

    /// 個別リポジトリの妥当性チェック
    fn validate_repository(&self, repo: &ManifestRepo) -> Result<(), String> {
        if repo.url.is_empty() {
            return Err("Repository URL cannot be empty".to_string());
        }

        if repo.dest.is_empty() {
            return Err("Repository destination cannot be empty".to_string());
        }

        // SCMタイプとURLの整合性チェック
        if !repo.scm.is_valid_url_scheme(&repo.url) {
            return Err(format!(
                "URL '{}' is not valid for SCM type '{}'", 
                repo.url, repo.scm
            ));
        }

        Ok(())
    }
}

impl Default for WorkspaceConfig {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_config_creation() {
        let config = WorkspaceConfig::new()
            .with_defaults(Defaults {
                scm: ScmType::Git,
                branch: Some("main".to_string()),
                remote: Some("origin".to_string()),
                shallow: false,
            });

        assert!(config.defaults.is_some());
        assert_eq!(config.defaults.unwrap().scm, ScmType::Git);
    }

    #[test]
    fn test_repository_group() {
        let repos = vec![
            ManifestRepo::new("https://github.com/user/repo1.git", "repo1"),
            ManifestRepo::new("https://github.com/user/repo2.git", "repo2"),
        ];

        let group = RepositoryGroup::new("test-group", repos)
            .with_description("Test repositories");

        assert_eq!(group.name, "test-group");
        assert_eq!(group.description, Some("Test repositories".to_string()));
        assert_eq!(group.repositories.len(), 2);
    }

    #[test]
    fn test_defaults_application() {
        let defaults = Defaults {
            scm: ScmType::Git,
            branch: Some("develop".to_string()),
            remote: Some("upstream".to_string()),
            shallow: true,
        };

        let mut repo = ManifestRepo::new("https://github.com/user/repo.git", "repo");
        // SCMはデフォルトのGitのまま、ブランチは未設定

        let config = WorkspaceConfig::new()
            .with_defaults(defaults)
            .with_repositories(vec![repo]);

        let normalized = config.normalize_repositories();
        assert_eq!(normalized[0].scm, ScmType::Git);
        assert_eq!(normalized[0].branch, Some("develop".to_string()));
    }

    #[test]
    fn test_mixed_scm_validation() {
        let repos = vec![
            ManifestRepo::with_scm("https://github.com/user/repo.git", "git-repo", ScmType::Git),
            ManifestRepo::with_scm("https://svn.server.com/repos/project", "svn-repo", ScmType::Svn),
            ManifestRepo::with_scm("perforce://p4:1666//depot/project/...", "p4-repo", ScmType::P4),
        ];

        let config = WorkspaceConfig::new().with_repositories(repos);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_invalid_url_validation() {
        let repos = vec![
            ManifestRepo::with_scm("https://github.com/user/repo.git", "repo", ScmType::P4), // 不正な組み合わせ
        ];

        let config = WorkspaceConfig::new().with_repositories(repos);
        assert!(config.validate().is_err());
    }
}