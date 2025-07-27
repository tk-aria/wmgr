use super::repository::{Remote, Repository};
use crate::domain::value_objects::scm_type::ScmType;
use crate::infrastructure::scm::{CloneOptions, SyncOptions};
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

    /// SCM固有のオプション
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scm_options: Option<ScmOptions>,

    /// リビジョン/コミット/チェンジリスト（SCM共通）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revision: Option<String>,

    /// 認証ユーザー名（SVN/P4用）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,

    /// 認証パスワード（SVN/P4用、通常は環境変数）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,

    /// 追加のSCM固有オプション
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra_options: Option<Vec<String>>,
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

/// SCM固有のオプション
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ScmOptions {
    /// Git固有のオプション
    Git {
        /// リモート名（デフォルト: origin）
        #[serde(skip_serializing_if = "Option::is_none")]
        remote: Option<String>,
        
        /// 浅いクローンの深度（デフォルト: 1）
        #[serde(skip_serializing_if = "Option::is_none")]
        depth: Option<u32>,
        
        /// サブモジュールも再帰的にクローンするか
        #[serde(default)]
        recurse_submodules: bool,
    },
    
    /// SVN固有のオプション
    Svn {
        /// 特定のリビジョン
        #[serde(skip_serializing_if = "Option::is_none")]
        revision: Option<String>,
        
        /// ユーザー名
        #[serde(skip_serializing_if = "Option::is_none")]
        username: Option<String>,
        
        /// パスワード（通常は環境変数で指定）
        #[serde(skip_serializing_if = "Option::is_none")]
        password: Option<String>,
    },
    
    /// Perforce固有のオプション
    P4 {
        /// クライアントワークスペース名
        #[serde(skip_serializing_if = "Option::is_none")]
        client: Option<String>,
        
        /// 特定のチェンジリスト
        #[serde(skip_serializing_if = "Option::is_none")]
        changelist: Option<String>,
        
        /// ストリーム（P4 Streams使用時）
        #[serde(skip_serializing_if = "Option::is_none")]
        stream: Option<String>,
        
        /// ユーザー名
        #[serde(skip_serializing_if = "Option::is_none")]
        username: Option<String>,
        
        /// パスワード（通常は環境変数で指定）
        #[serde(skip_serializing_if = "Option::is_none")]
        password: Option<String>,
    },
}

impl Default for ScmOptions {
    fn default() -> Self {
        ScmOptions::Git {
            remote: None,
            depth: None,
            recurse_submodules: false,
        }
    }
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
            scm_options: None,
            revision: None,
            username: None,
            password: None,
            extra_options: None,
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
            scm_options: None,
            revision: None,
            username: None,
            password: None,
            extra_options: None,
        }
    }

    /// ブランチを設定
    pub fn with_branch(mut self, branch: impl Into<String>) -> Self {
        self.branch = Some(branch.into());
        self
    }

    /// リビジョンを設定
    pub fn with_revision(mut self, revision: impl Into<String>) -> Self {
        self.revision = Some(revision.into());
        self
    }

    /// 認証情報を設定
    pub fn with_auth(mut self, username: impl Into<String>, password: impl Into<String>) -> Self {
        self.username = Some(username.into());
        self.password = Some(password.into());
        self
    }

    /// SCM固有オプションを設定
    pub fn with_scm_options(mut self, options: ScmOptions) -> Self {
        self.scm_options = Some(options);
        self
    }

    /// 追加オプションを設定
    pub fn with_extra_options(mut self, options: Vec<String>) -> Self {
        self.extra_options = Some(options);
        self
    }

    /// SCM固有の設定を取得（Git用）
    pub fn get_git_options(&self) -> Option<&ScmOptions> {
        match (&self.scm, &self.scm_options) {
            (ScmType::Git, Some(ScmOptions::Git { .. })) => self.scm_options.as_ref(),
            _ => None,
        }
    }

    /// SCM固有の設定を取得（SVN用）
    pub fn get_svn_options(&self) -> Option<&ScmOptions> {
        match (&self.scm, &self.scm_options) {
            (ScmType::Svn, Some(ScmOptions::Svn { .. })) => self.scm_options.as_ref(),
            _ => None,
        }
    }

    /// SCM固有の設定を取得（P4用）
    pub fn get_p4_options(&self) -> Option<&ScmOptions> {
        match (&self.scm, &self.scm_options) {
            (ScmType::P4, Some(ScmOptions::P4 { .. })) => self.scm_options.as_ref(),
            _ => None,
        }
    }

    /// 有効なリビジョンを取得（SCM固有オプションまたは共通フィールドから）
    pub fn get_effective_revision(&self) -> Option<&String> {
        // まずSCM固有オプションをチェック
        match &self.scm_options {
            Some(ScmOptions::Svn { revision, .. }) if revision.is_some() => revision.as_ref(),
            Some(ScmOptions::P4 { changelist, .. }) if changelist.is_some() => changelist.as_ref(),
            _ => self.revision.as_ref(), // フォールバックとして共通フィールドを使用
        }
    }

    /// 有効な認証情報を取得
    pub fn get_effective_auth(&self) -> (Option<&String>, Option<&String>) {
        // まずSCM固有オプションをチェック
        match &self.scm_options {
            Some(ScmOptions::Svn { username, password, .. }) => {
                (username.as_ref(), password.as_ref())
            }
            Some(ScmOptions::P4 { username, password, .. }) => {
                (username.as_ref(), password.as_ref())
            }
            _ => (self.username.as_ref(), self.password.as_ref()),
        }
    }

    /// CloneOptionsに変換
    pub fn to_clone_options(&self) -> CloneOptions {
        let (username, password) = self.get_effective_auth();
        let mut options = CloneOptions {
            branch: self.branch.clone(),
            shallow: self.shallow,
            revision: self.get_effective_revision().cloned(),
            username: username.cloned(),
            password: password.cloned(),
            extra_options: self.extra_options.clone().unwrap_or_default(),
            ..Default::default()
        };

        // SCM固有オプションから設定を取得
        match &self.scm_options {
            Some(ScmOptions::Git { remote, depth, recurse_submodules }) => {
                options.remote = remote.clone();
                options.depth = *depth;
                options.recurse_submodules = *recurse_submodules;
            }
            Some(ScmOptions::P4 { client, stream, .. }) => {
                options.client = client.clone();
                options.stream = stream.clone();
            }
            _ => {}
        }

        options
    }

    /// SyncOptionsに変換
    pub fn to_sync_options(&self, force: bool) -> SyncOptions {
        let (username, password) = self.get_effective_auth();
        let mut options = SyncOptions {
            branch: self.branch.clone(),
            force,
            revision: self.get_effective_revision().cloned(),
            username: username.cloned(),
            password: password.cloned(),
            extra_options: self.extra_options.clone().unwrap_or_default(),
            ..Default::default()
        };

        // SCM固有オプションから設定を取得
        match &self.scm_options {
            Some(ScmOptions::P4 { client, stream, .. }) => {
                options.client = client.clone();
                options.stream = stream.clone();
            }
            _ => {}
        }

        options
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
