use serde::{Deserialize, Serialize};

/// リモートリポジトリの情報
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Remote {
    /// リモート名（例: origin）
    pub name: String,
    /// リモートのURL
    pub url: String,
}

impl Remote {
    /// 新しいRemoteインスタンスを作成
    pub fn new(name: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            url: url.into(),
        }
    }
}

/// リポジトリエンティティ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    /// ワークスペース内でのリポジトリの相対パス
    pub dest: String,

    /// リモートリポジトリのリスト
    pub remotes: Vec<Remote>,

    /// 対象ブランチ名
    pub branch: Option<String>,

    /// 元のブランチ名（同期前のブランチ）
    pub orig_branch: Option<String>,

    /// ブランチを維持するかどうか
    pub keep_branch: bool,

    /// デフォルトブランチかどうか
    pub is_default_branch: bool,

    /// 固定されたSHA1ハッシュ
    pub sha1: Option<String>,

    /// 完全なSHA1ハッシュ
    pub sha1_full: Option<String>,

    /// タグ名
    pub tag: Option<String>,

    /// shallow cloneを使用するか
    pub shallow: bool,

    /// ベアリポジトリかどうか
    pub is_bare: bool,
}

impl Repository {
    /// 新しいRepositoryインスタンスを作成
    pub fn new(dest: impl Into<String>, remotes: Vec<Remote>) -> Self {
        Self {
            dest: dest.into(),
            remotes,
            branch: None,
            orig_branch: None,
            keep_branch: false,
            is_default_branch: false,
            sha1: None,
            sha1_full: None,
            tag: None,
            shallow: false,
            is_bare: false,
        }
    }

    /// ブランチを設定
    pub fn with_branch(mut self, branch: impl Into<String>) -> Self {
        self.branch = Some(branch.into());
        self
    }

    /// SHA1を設定
    pub fn with_sha1(mut self, sha1: impl Into<String>) -> Self {
        self.sha1 = Some(sha1.into());
        self
    }

    /// タグを設定
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tag = Some(tag.into());
        self
    }

    /// shallow cloneを有効化
    pub fn with_shallow(mut self, shallow: bool) -> Self {
        self.shallow = shallow;
        self
    }

    /// クローン用のURLを取得（最初のリモートのURL）
    pub fn clone_url(&self) -> Option<&str> {
        self.remotes.first().map(|r| r.url.as_str())
    }

    /// 特定の名前のリモートを取得
    pub fn get_remote(&self, name: &str) -> Option<&Remote> {
        self.remotes.iter().find(|r| r.name == name)
    }

    /// デフォルトのリモート（origin）を取得
    pub fn get_origin(&self) -> Option<&Remote> {
        self.get_remote("origin")
    }

    /// リポジトリが固定参照（SHA1またはタグ）を持っているか
    pub fn has_fixed_ref(&self) -> bool {
        self.sha1.is_some() || self.tag.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remote_creation() {
        let remote = Remote::new("origin", "git@github.com:example/repo.git");
        assert_eq!(remote.name, "origin");
        assert_eq!(remote.url, "git@github.com:example/repo.git");
    }

    #[test]
    fn test_repository_creation() {
        let remotes = vec![Remote::new("origin", "git@github.com:example/repo.git")];
        let repo = Repository::new("path/to/repo", remotes);

        assert_eq!(repo.dest, "path/to/repo");
        assert_eq!(repo.remotes.len(), 1);
        assert_eq!(repo.clone_url(), Some("git@github.com:example/repo.git"));
    }

    #[test]
    fn test_repository_builder() {
        let remotes = vec![Remote::new("origin", "git@github.com:example/repo.git")];
        let repo = Repository::new("path/to/repo", remotes)
            .with_branch("main")
            .with_sha1("abc123")
            .with_shallow(true);

        assert_eq!(repo.branch, Some("main".to_string()));
        assert_eq!(repo.sha1, Some("abc123".to_string()));
        assert!(repo.shallow);
        assert!(repo.has_fixed_ref());
    }

    #[test]
    fn test_get_remote() {
        let remotes = vec![
            Remote::new("origin", "git@github.com:example/repo.git"),
            Remote::new("upstream", "git@github.com:upstream/repo.git"),
        ];
        let repo = Repository::new("path/to/repo", remotes);

        assert!(repo.get_remote("origin").is_some());
        assert!(repo.get_remote("upstream").is_some());
        assert!(repo.get_remote("nonexistent").is_none());

        let origin = repo.get_origin().unwrap();
        assert_eq!(origin.name, "origin");
    }
}
