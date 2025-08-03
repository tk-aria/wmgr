use super::{manifest::Manifest, repository::Repository};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// ワークスペースの設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    /// マニフェストリポジトリのURL
    pub manifest_url: String,

    /// マニフェストのブランチ
    pub manifest_branch: String,

    /// shallow cloneを使用するか
    #[serde(default)]
    pub shallow_clones: bool,

    /// 使用するリポジトリグループのリスト
    #[serde(default)]
    pub repo_groups: Vec<String>,

    /// 全てのリポジトリをクローンするか（グループを無視）
    #[serde(default)]
    pub clone_all_repos: bool,

    /// 単一リモート名（設定されている場合）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub singular_remote: Option<String>,
}

impl WorkspaceConfig {
    /// 新しいWorkspaceConfigインスタンスを作成
    pub fn new(manifest_url: impl Into<String>, manifest_branch: impl Into<String>) -> Self {
        Self {
            manifest_url: manifest_url.into(),
            manifest_branch: manifest_branch.into(),
            shallow_clones: false,
            repo_groups: vec!["default".to_string()],
            clone_all_repos: false,
            singular_remote: None,
        }
    }

    /// デフォルトのWorkspaceConfigインスタンスを作成（ローカルファイル用）
    pub fn default_local() -> Self {
        Self {
            manifest_url: "".to_string(),
            manifest_branch: "main".to_string(),
            shallow_clones: false,
            repo_groups: vec!["default".to_string()],
            clone_all_repos: false,
            singular_remote: None,
        }
    }

    /// リポジトリグループを設定
    pub fn with_repo_groups(mut self, groups: Vec<String>) -> Self {
        self.repo_groups = groups;
        self
    }

    /// shallow cloneを有効化
    pub fn with_shallow_clones(mut self, shallow: bool) -> Self {
        self.shallow_clones = shallow;
        self
    }

    /// 全リポジトリクローンを有効化
    pub fn with_clone_all_repos(mut self, clone_all: bool) -> Self {
        self.clone_all_repos = clone_all;
        self
    }

    /// 単一リモートを設定
    pub fn with_singular_remote(mut self, remote: impl Into<String>) -> Self {
        self.singular_remote = Some(remote.into());
        self
    }

    /// デフォルトグループのみを使用しているか
    pub fn is_using_default_group(&self) -> bool {
        self.repo_groups.len() == 1 && self.repo_groups[0] == "default"
    }
}

/// ワークスペースの状態
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceStatus {
    /// 未初期化（ワークスペースディレクトリが存在しない）
    Uninitialized,
    /// 初期化済み（設定ファイルが存在する）
    Initialized,
    /// 破損（設定ファイルが不正など）
    Corrupted,
}

/// wmgrワークスペースのエンティティ
#[derive(Debug, Clone)]
pub struct Workspace {
    /// ワークスペースのルートパス
    pub root_path: PathBuf,

    /// ワークスペースの設定
    pub config: WorkspaceConfig,

    /// 現在のマニフェスト
    pub manifest: Option<Manifest>,

    /// ワークスペース内のリポジトリリスト
    pub repositories: Vec<Repository>,

    /// ワークスペースの状態
    pub status: WorkspaceStatus,
}

impl Workspace {
    /// 新しいWorkspaceインスタンスを作成
    pub fn new(root_path: PathBuf, config: WorkspaceConfig) -> Self {
        Self {
            root_path,
            config,
            manifest: None,
            repositories: Vec::new(),
            status: WorkspaceStatus::Uninitialized,
        }
    }

    /// マニフェストを設定
    pub fn with_manifest(mut self, manifest: Manifest) -> Self {
        self.manifest = Some(manifest);
        self
    }

    /// リポジトリリストを設定
    pub fn with_repositories(mut self, repositories: Vec<Repository>) -> Self {
        self.repositories = repositories;
        self
    }

    /// ワークスペースの状態を設定
    pub fn with_status(mut self, status: WorkspaceStatus) -> Self {
        self.status = status;
        self
    }

    /// .tsrcディレクトリのパスを取得
    pub fn tsrc_dir(&self) -> PathBuf {
        self.root_path.join(".tsrc")
    }

    /// config.ymlファイルのパスを取得
    pub fn config_path(&self) -> PathBuf {
        self.tsrc_dir().join("config.yml")
    }

    /// マニフェストディレクトリのパスを取得
    /// Note: For local-first implementation, this points to .tsrc directory
    pub fn manifest_dir(&self) -> PathBuf {
        self.tsrc_dir()
    }

    /// マニフェストファイル（wmgr.yml、manifest.yml）のパスを取得
    /// walkdir + regex を使用した動的探索で優先順位付き
    pub fn manifest_file_path(&self) -> PathBuf {
        let found_files = self.find_manifest_files_with_regex();

        if let Some(first_file) = found_files.first() {
            first_file.clone()
        } else {
            // デフォルトはカレントディレクトリのwmgr.yml
            self.root_path.join("wmgr.yml")
        }
    }

    /// walkdir + regex でマニフェストファイルを動的に探索
    /// 優先順位でソートされたリストを返す
    pub fn find_manifest_files_with_regex(&self) -> Vec<std::path::PathBuf> {
        use regex::Regex;
        use walkdir::WalkDir;

        let manifest_regex = Regex::new(r"^(wmgr|manifest)\.(yml|yaml)$").unwrap();

        let search_dirs = vec![
            self.root_path.clone(),       // カレントディレクトリ
            self.root_path.join(".wmgr"), // .wmgrディレクトリ
        ];

        let mut manifest_files = Vec::new();

        for search_dir in search_dirs {
            if !search_dir.exists() {
                continue;
            }

            // walkdirで探索（深度1のみ = 直下のファイルのみ）
            for entry in WalkDir::new(&search_dir)
                .max_depth(1) // 直下のファイルのみ
                .into_iter()
                .filter_map(|e| e.ok()) // エラーを除外
                .filter(|e| e.file_type().is_file())
            // ファイルのみ
            {
                // ファイル名取得
                if let Some(file_name) = entry.file_name().to_str() {
                    // 正規表現でマッチチェック
                    if manifest_regex.is_match(file_name) {
                        manifest_files.push(entry.path().to_path_buf());
                    }
                }
            }
        }

        // 優先順位でソート
        manifest_files.sort_by_key(|path| Self::get_file_priority(path));

        manifest_files
    }

    /// ファイルの優先順位を取得（数値が小さいほど優先度が高い）
    fn get_file_priority(path: &std::path::PathBuf) -> u8 {
        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        let is_in_wmgr = path
            .parent()
            .and_then(|p| p.file_name())
            .map(|n| n == ".wmgr")
            .unwrap_or(false);

        match (file_name, is_in_wmgr) {
            ("wmgr.yml", false) => 1,
            ("wmgr.yaml", false) => 2,
            ("manifest.yml", false) => 3,
            ("manifest.yaml", false) => 4,
            ("wmgr.yml", true) => 5,
            ("wmgr.yaml", true) => 6,
            ("manifest.yml", true) => 7,
            ("manifest.yaml", true) => 8,
            _ => 99,
        }
    }

    /// 旧マニフェストファイル（.tsrc/manifest.yml）のパスを取得
    /// レガシーサポート用
    pub fn legacy_manifest_file_path(&self) -> PathBuf {
        self.tsrc_dir().join("manifest.yml")
    }

    /// 特定のリポジトリのパスを取得
    pub fn repo_path(&self, dest: &str) -> PathBuf {
        self.root_path.join(dest)
    }

    /// ワークスペースが初期化されているか
    pub fn is_initialized(&self) -> bool {
        matches!(self.status, WorkspaceStatus::Initialized)
    }

    /// ワークスペースが破損しているか
    pub fn is_corrupted(&self) -> bool {
        matches!(self.status, WorkspaceStatus::Corrupted)
    }

    /// 設定されたグループに基づいてリポジトリをフィルタリング
    pub fn filter_repos_by_groups(&self) -> Vec<&Repository> {
        if let Some(manifest) = &self.manifest {
            if self.config.clone_all_repos {
                // 全リポジトリを返す
                self.repositories.iter().collect()
            } else if self.config.repo_groups.is_empty() || self.config.is_using_default_group() {
                // デフォルトグループまたはグループ未指定の場合、全リポジトリを返す
                self.repositories.iter().collect()
            } else {
                // 指定されたグループのリポジトリのみを返す
                let mut filtered_repos = Vec::new();
                for group_name in &self.config.repo_groups {
                    let group_repos = manifest.get_repos_in_group(group_name);
                    for manifest_repo in group_repos {
                        if let Some(repo) = self
                            .repositories
                            .iter()
                            .find(|r| r.dest == manifest_repo.dest)
                        {
                            if !filtered_repos
                                .iter()
                                .any(|r: &&Repository| r.dest == repo.dest)
                            {
                                filtered_repos.push(repo);
                            }
                        }
                    }
                }
                filtered_repos
            }
        } else {
            // マニフェストがない場合は全リポジトリを返す
            self.repositories.iter().collect()
        }
    }

    /// 特定のdestでリポジトリを検索
    pub fn find_repository(&self, dest: &str) -> Option<&Repository> {
        self.repositories.iter().find(|r| r.dest == dest)
    }

    /// 特定のdestでリポジトリを検索（mutable）
    pub fn find_repository_mut(&mut self, dest: &str) -> Option<&mut Repository> {
        self.repositories.iter_mut().find(|r| r.dest == dest)
    }

    /// 現在のディレクトリから上位に向かってワークスペースルートを発見
    pub fn discover_workspace_root(start_path: &std::path::Path) -> Option<PathBuf> {
        let mut current_path = start_path.to_path_buf();
        
        loop {
            // 現在のパスで一時的なワークスペースを作成してマニフェストファイルを探索
            let temp_workspace = Self::new(current_path.clone(), WorkspaceConfig::default_local());
            let manifest_files = temp_workspace.find_manifest_files_with_regex();
            
            if !manifest_files.is_empty() {
                return Some(current_path);
            }
            
            // 親ディレクトリに移動
            if let Some(parent) = current_path.parent() {
                current_path = parent.to_path_buf();
            } else {
                // ルートディレクトリに到達したが見つからない
                break;
            }
        }
        
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::{
        manifest::{Group, ManifestRepo},
        repository::Remote,
    };
    use std::collections::HashMap;

    #[test]
    fn test_workspace_config_creation() {
        let config = WorkspaceConfig::new("git@github.com:example/manifest.git", "main")
            .with_repo_groups(vec!["group1".to_string(), "group2".to_string()])
            .with_shallow_clones(true);

        assert_eq!(config.manifest_url, "git@github.com:example/manifest.git");
        assert_eq!(config.manifest_branch, "main");
        assert!(config.shallow_clones);
        assert_eq!(config.repo_groups.len(), 2);
    }

    #[test]
    fn test_workspace_paths() {
        let config = WorkspaceConfig::new("git@github.com:example/manifest.git", "main");
        let workspace = Workspace::new(PathBuf::from("/path/to/workspace"), config);

        assert_eq!(
            workspace.tsrc_dir(),
            PathBuf::from("/path/to/workspace/.tsrc")
        );
        assert_eq!(
            workspace.config_path(),
            PathBuf::from("/path/to/workspace/.tsrc/config.yml")
        );
        assert_eq!(
            workspace.manifest_dir(),
            PathBuf::from("/path/to/workspace/.tsrc")
        );
        assert_eq!(
            workspace.repo_path("repo1"),
            PathBuf::from("/path/to/workspace/repo1")
        );
    }

    #[test]
    fn test_workspace_status() {
        let config = WorkspaceConfig::new("git@github.com:example/manifest.git", "main");
        let mut workspace = Workspace::new(PathBuf::from("/path/to/workspace"), config);

        assert!(!workspace.is_initialized());
        assert!(!workspace.is_corrupted());

        workspace.status = WorkspaceStatus::Initialized;
        assert!(workspace.is_initialized());

        workspace.status = WorkspaceStatus::Corrupted;
        assert!(workspace.is_corrupted());
    }

    #[test]
    fn test_filter_repos_by_groups() {
        let config = WorkspaceConfig::new("git@github.com:example/manifest.git", "main")
            .with_repo_groups(vec!["group1".to_string()]);

        // マニフェストとリポジトリを設定
        let manifest_repos = vec![
            ManifestRepo::new("git@github.com:example/repo1.git", "repo1"),
            ManifestRepo::new("git@github.com:example/repo2.git", "repo2"),
        ];

        let mut groups = HashMap::new();
        groups.insert("group1".to_string(), Group::new(vec!["repo1".to_string()]));

        let manifest = Manifest::new(manifest_repos).with_groups(groups);

        let repositories = vec![
            Repository::new(
                "repo1",
                vec![Remote::new("origin", "git@github.com:example/repo1.git")],
            ),
            Repository::new(
                "repo2",
                vec![Remote::new("origin", "git@github.com:example/repo2.git")],
            ),
        ];

        let workspace = Workspace::new(PathBuf::from("/path/to/workspace"), config)
            .with_manifest(manifest)
            .with_repositories(repositories);

        let filtered = workspace.filter_repos_by_groups();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].dest, "repo1");
    }

    #[test]
    fn test_workspace_discovery() {
        use tempfile::TempDir;
        
        // テスト用の一時ディレクトリを作成
        let temp_dir = TempDir::new().unwrap();
        let workspace_root = temp_dir.path().to_path_buf();
        
        // wmgr.ymlファイルを作成
        let manifest_file = workspace_root.join("wmgr.yml");
        std::fs::write(&manifest_file, "repos: []").unwrap();
        
        // サブディレクトリを作成
        let sub_dir = workspace_root.join("sub").join("dir");
        std::fs::create_dir_all(&sub_dir).unwrap();
        
        // サブディレクトリからワークスペースルートを発見
        let discovered = Workspace::discover_workspace_root(&sub_dir);
        assert!(discovered.is_some());
        assert_eq!(discovered.unwrap(), workspace_root);
        
        // マニフェストファイルがない場合はNoneを返す
        let temp_dir2 = TempDir::new().unwrap();
        let no_manifest_dir = temp_dir2.path().join("no_manifest");
        std::fs::create_dir_all(&no_manifest_dir).unwrap();
        
        let not_found = Workspace::discover_workspace_root(&no_manifest_dir);
        assert!(not_found.is_none());
    }
}
