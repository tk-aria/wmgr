use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use super::{repository::Repository, manifest::Manifest};

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

/// tsrcワークスペースのエンティティ
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
    
    /// マニフェストファイル（manifest.yml）のパスを取得
    /// 優先順位: カレントディレクトリのmanifest.yml → .wmgr/manifest.yml
    pub fn manifest_file_path(&self) -> PathBuf {
        let current_manifest = self.root_path.join("manifest.yml");
        let wmgr_manifest = self.root_path.join(".wmgr").join("manifest.yml");
        
        if current_manifest.exists() {
            current_manifest
        } else if wmgr_manifest.exists() {
            wmgr_manifest
        } else {
            // デフォルトはカレントディレクトリのmanifest.yml
            current_manifest
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
                        if let Some(repo) = self.repositories.iter().find(|r| r.dest == manifest_repo.dest) {
                            if !filtered_repos.iter().any(|r: &&Repository| r.dest == repo.dest) {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use crate::domain::entities::{
        manifest::{Group, ManifestRepo},
        repository::Remote,
    };
    
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
        
        assert_eq!(workspace.tsrc_dir(), PathBuf::from("/path/to/workspace/.tsrc"));
        assert_eq!(workspace.config_path(), PathBuf::from("/path/to/workspace/.tsrc/config.yml"));
        assert_eq!(workspace.manifest_dir(), PathBuf::from("/path/to/workspace/.tsrc"));
        assert_eq!(workspace.repo_path("repo1"), PathBuf::from("/path/to/workspace/repo1"));
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
            Repository::new("repo1", vec![Remote::new("origin", "git@github.com:example/repo1.git")]),
            Repository::new("repo2", vec![Remote::new("origin", "git@github.com:example/repo2.git")]),
        ];
        
        let workspace = Workspace::new(PathBuf::from("/path/to/workspace"), config)
            .with_manifest(manifest)
            .with_repositories(repositories);
        
        let filtered = workspace.filter_repos_by_groups();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].dest, "repo1");
    }
}