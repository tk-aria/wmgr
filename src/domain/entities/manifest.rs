use super::repository::{Remote, Repository};
use crate::domain::value_objects::scm_type::ScmType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// グループの定義
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Group {
    /// グループに含まれるリポジトリのdest（相対パス）のリスト
    pub repos: Vec<String>,

    /// グループの説明（オプション）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl Group {
    /// 新しいGroupインスタンスを作成
    pub fn new(repos: Vec<String>) -> Self {
        Self {
            repos,
            description: None,
        }
    }

    /// 説明を設定
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

/// マニフェストのリポジトリ定義
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestRepo {
    /// リポジトリのURL
    pub url: String,

    /// ワークスペース内での相対パス
    pub dest: String,

    /// ブランチ名（オプション）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,

    /// SHA1ハッシュ（オプション）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha1: Option<String>,

    /// タグ（オプション）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,

    /// 追加のリモート定義
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remotes: Option<Vec<Remote>>,

    /// shallow cloneを使用するか
    #[serde(default)]
    pub shallow: bool,

    /// ファイルコピー操作の定義
    #[serde(skip_serializing_if = "Option::is_none")]
    pub copy: Option<Vec<FileCopy>>,

    /// シンボリックリンク操作の定義
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symlink: Option<Vec<FileSymlink>>,

    /// SCM (Source Control Management) の種別
    #[serde(default)]
    pub scm: ScmType,
}

/// ファイルコピー操作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileCopy {
    /// コピー元のファイル（リポジトリ内の相対パス）
    pub file: String,

    /// コピー先のパス（ワークスペースルートからの相対パス）
    pub dest: String,
}

/// シンボリックリンク操作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSymlink {
    /// リンク元（ワークスペースルートからの相対パス）
    pub source: String,

    /// リンク先（sourceからの相対パス）
    pub target: String,
}

impl ManifestRepo {
    /// 新しいManifestRepoインスタンスを作成
    pub fn new(url: impl Into<String>, dest: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            dest: dest.into(),
            branch: None,
            sha1: None,
            tag: None,
            remotes: None,
            shallow: false,
            copy: None,
            symlink: None,
            scm: ScmType::default(),
        }
    }

    /// SCM種別を指定して新しいManifestRepoインスタンスを作成
    pub fn with_scm(url: impl Into<String>, dest: impl Into<String>, scm: ScmType) -> Self {
        Self {
            url: url.into(),
            dest: dest.into(),
            branch: None,
            sha1: None,
            tag: None,
            remotes: None,
            shallow: false,
            copy: None,
            symlink: None,
            scm,
        }
    }

    /// Repositoryエンティティに変換
    pub fn to_repository(&self) -> Repository {
        let mut remotes = vec![Remote::new("origin", &self.url)];

        // 追加のリモートがあれば追加
        if let Some(additional_remotes) = &self.remotes {
            remotes.extend(additional_remotes.iter().cloned());
        }

        let mut repo = Repository::with_scm(&self.dest, remotes, self.scm.clone());

        if let Some(branch) = &self.branch {
            repo = repo.with_branch(branch);
        }
        if let Some(sha1) = &self.sha1 {
            repo = repo.with_sha1(sha1);
        }
        if let Some(tag) = &self.tag {
            repo = repo.with_tag(tag);
        }
        repo = repo.with_shallow(self.shallow);

        repo
    }
}

/// マニフェストファイルの構造
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    /// リポジトリのリスト
    pub repos: Vec<ManifestRepo>,

    /// グループ定義（オプション）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub groups: Option<HashMap<String, Group>>,

    /// デフォルトブランチ（オプション）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_branch: Option<String>,

    /// デフォルトのSCM種別（オプション）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_scm: Option<ScmType>,
}

impl Manifest {
    /// 新しいManifestインスタンスを作成
    pub fn new(repos: Vec<ManifestRepo>) -> Self {
        Self {
            repos,
            groups: None,
            default_branch: None,
            default_scm: None,
        }
    }

    /// グループを設定
    pub fn with_groups(mut self, groups: HashMap<String, Group>) -> Self {
        self.groups = Some(groups);
        self
    }

    /// デフォルトブランチを設定
    pub fn with_default_branch(mut self, branch: impl Into<String>) -> Self {
        self.default_branch = Some(branch.into());
        self
    }

    /// デフォルトSCM種別を設定
    pub fn with_default_scm(mut self, scm: ScmType) -> Self {
        self.default_scm = Some(scm);
        self
    }

    /// 特定のグループに属するリポジトリを取得
    pub fn get_repos_in_group(&self, group_name: &str) -> Vec<&ManifestRepo> {
        if let Some(groups) = &self.groups {
            if let Some(group) = groups.get(group_name) {
                self.repos
                    .iter()
                    .filter(|repo| group.repos.contains(&repo.dest))
                    .collect()
            } else {
                vec![]
            }
        } else {
            vec![]
        }
    }

    /// 全てのリポジトリをRepositoryエンティティのリストに変換
    pub fn to_repositories(&self) -> Vec<Repository> {
        self.repos.iter().map(|r| r.to_repository()).collect()
    }

    /// destでリポジトリを検索
    pub fn find_repo_by_dest(&self, dest: &str) -> Option<&ManifestRepo> {
        self.repos.iter().find(|r| r.dest == dest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_group_creation() {
        let group = Group::new(vec!["repo1".to_string(), "repo2".to_string()])
            .with_description("Test group");

        assert_eq!(group.repos.len(), 2);
        assert_eq!(group.description, Some("Test group".to_string()));
    }

    #[test]
    fn test_manifest_repo_creation() {
        let repo = ManifestRepo::new("git@github.com:example/repo.git", "path/to/repo");

        assert_eq!(repo.url, "git@github.com:example/repo.git");
        assert_eq!(repo.dest, "path/to/repo");
        assert!(!repo.shallow);
    }

    #[test]
    fn test_manifest_repo_to_repository() {
        let mut manifest_repo =
            ManifestRepo::new("git@github.com:example/repo.git", "path/to/repo");
        manifest_repo.branch = Some("main".to_string());
        manifest_repo.sha1 = Some("abc123".to_string());

        let repo = manifest_repo.to_repository();

        assert_eq!(repo.dest, "path/to/repo");
        assert_eq!(repo.branch, Some("main".to_string()));
        assert_eq!(repo.sha1, Some("abc123".to_string()));
        assert_eq!(repo.clone_url(), Some("git@github.com:example/repo.git"));
    }

    #[test]
    fn test_manifest_with_groups() {
        let repos = vec![
            ManifestRepo::new("git@github.com:example/repo1.git", "repo1"),
            ManifestRepo::new("git@github.com:example/repo2.git", "repo2"),
            ManifestRepo::new("git@github.com:example/repo3.git", "repo3"),
        ];

        let mut groups = HashMap::new();
        groups.insert(
            "group1".to_string(),
            Group::new(vec!["repo1".to_string(), "repo2".to_string()]),
        );

        let manifest = Manifest::new(repos).with_groups(groups);

        let group_repos = manifest.get_repos_in_group("group1");
        assert_eq!(group_repos.len(), 2);
        assert_eq!(group_repos[0].dest, "repo1");
        assert_eq!(group_repos[1].dest, "repo2");
    }
}
