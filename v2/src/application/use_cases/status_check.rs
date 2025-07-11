use std::path::PathBuf;
use thiserror::Error;
use serde::{Deserialize, Serialize};
use crate::domain::entities::{
    workspace::Workspace, 
    manifest::ManifestRepo
};
use crate::infrastructure::git::repository::{GitRepository, GitRepositoryError};

/// StatusCheck関連のエラー
#[derive(Debug, Error)]
pub enum StatusCheckError {
    #[error("Workspace not initialized: {0}")]
    WorkspaceNotInitialized(String),
    
    #[error("Git status check failed for repo '{repo}': {error}")]
    GitStatusFailed { repo: String, error: String },
    
    #[error("Repository not found: {0}")]
    RepositoryNotFound(String),
    
    #[error("Git operation failed: {0}")]
    GitOperationFailed(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Branch name error: {0}")]
    BranchNameError(#[from] crate::domain::value_objects::branch_name::BranchNameError),
    
    #[error("Git repository error: {0}")]
    GitRepositoryError(#[from] GitRepositoryError),
}

/// ステータス確認の設定
#[derive(Debug, Clone)]
pub struct StatusCheckConfig {
    /// 特定のグループのみをチェックするか（Noneの場合は全て）
    pub groups: Option<Vec<String>>,
    
    /// ブランチ情報を表示するか
    pub show_branch: bool,
    
    /// コンパクト表示を使用するか
    pub compact: bool,
    
    /// 詳細ログを出力するか
    pub verbose: bool,
}

impl Default for StatusCheckConfig {
    fn default() -> Self {
        Self {
            groups: None,
            show_branch: false,
            compact: false,
            verbose: false,
        }
    }
}

/// リポジトリの状態
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RepositoryState {
    /// クリーン（変更なし）
    Clean,
    /// ダーティ（未コミットの変更あり）
    Dirty,
    /// 存在しない
    Missing,
    /// ブランチが期待と異なる
    WrongBranch,
    /// リモートと差分あり
    OutOfSync,
    /// エラー状態
    Error,
}

/// 単一リポジトリのステータス
#[derive(Debug, Clone)]
pub struct RepositoryStatus {
    /// リポジトリの相対パス
    pub dest: String,
    
    /// リポジトリの状態
    pub state: RepositoryState,
    
    /// 現在のブランチ
    pub current_branch: Option<String>,
    
    /// 期待されるブランチ
    pub expected_branch: Option<String>,
    
    /// 未追跡ファイル数
    pub untracked_files: usize,
    
    /// 変更されたファイル数
    pub modified_files: usize,
    
    /// ステージされたファイル数
    pub staged_files: usize,
    
    /// リモートより進んでいるコミット数
    pub commits_ahead: usize,
    
    /// リモートより遅れているコミット数
    pub commits_behind: usize,
    
    /// エラーメッセージ（エラー状態の場合）
    pub error_message: Option<String>,
}

impl RepositoryStatus {
    /// 新しいRepositoryStatusを作成
    pub fn new(dest: String) -> Self {
        Self {
            dest,
            state: RepositoryState::Missing,
            current_branch: None,
            expected_branch: None,
            untracked_files: 0,
            modified_files: 0,
            staged_files: 0,
            commits_ahead: 0,
            commits_behind: 0,
            error_message: None,
        }
    }
    
    /// エラー状態に設定
    pub fn with_error(mut self, error: String) -> Self {
        self.state = RepositoryState::Error;
        self.error_message = Some(error);
        self
    }
    
    /// ブランチ情報を設定
    pub fn with_branch_info(mut self, current: Option<String>, expected: Option<String>) -> Self {
        self.current_branch = current;
        self.expected_branch = expected;
        
        // ブランチが期待と異なる場合は状態を更新
        if let (Some(current), Some(expected)) = (&self.current_branch, &self.expected_branch) {
            if current != expected && self.state != RepositoryState::Error {
                self.state = RepositoryState::WrongBranch;
            }
        }
        
        self
    }
    
    /// ファイル変更情報を設定
    pub fn with_file_changes(mut self, untracked: usize, modified: usize, staged: usize) -> Self {
        self.untracked_files = untracked;
        self.modified_files = modified;
        self.staged_files = staged;
        
        // ファイル変更がある場合は状態を更新
        if (untracked + modified + staged) > 0 && self.state != RepositoryState::Error && self.state != RepositoryState::WrongBranch {
            self.state = RepositoryState::Dirty;
        } else if self.state != RepositoryState::Error && self.state != RepositoryState::WrongBranch && self.state != RepositoryState::Dirty {
            self.state = RepositoryState::Clean;
        }
        
        self
    }
    
    /// リモート差分情報を設定
    pub fn with_remote_diff(mut self, ahead: usize, behind: usize) -> Self {
        self.commits_ahead = ahead;
        self.commits_behind = behind;
        
        // リモートと差分がある場合は状態を更新
        if (ahead + behind) > 0 && self.state == RepositoryState::Clean {
            self.state = RepositoryState::OutOfSync;
        }
        
        self
    }
    
    /// 問題があるかチェック
    pub fn has_issues(&self) -> bool {
        !matches!(self.state, RepositoryState::Clean)
    }
}

/// 全体のステータス結果
#[derive(Debug, Clone)]
pub struct StatusResult {
    /// 各リポジトリのステータス
    pub repositories: Vec<RepositoryStatus>,
    
    /// 正常なリポジトリ数
    pub clean_count: usize,
    
    /// 問題のあるリポジトリ数
    pub dirty_count: usize,
    
    /// 見つからないリポジトリ数
    pub missing_count: usize,
    
    /// エラーのあるリポジトリ数
    pub error_count: usize,
}

impl StatusResult {
    pub fn new() -> Self {
        Self {
            repositories: Vec::new(),
            clean_count: 0,
            dirty_count: 0,
            missing_count: 0,
            error_count: 0,
        }
    }
    
    /// リポジトリステータスを追加
    pub fn add_repository(&mut self, status: RepositoryStatus) {
        match status.state {
            RepositoryState::Clean => self.clean_count += 1,
            RepositoryState::Missing => self.missing_count += 1,
            RepositoryState::Error => self.error_count += 1,
            _ => self.dirty_count += 1,
        }
        self.repositories.push(status);
    }
    
    /// 全体的に問題があるかチェック
    pub fn has_issues(&self) -> bool {
        self.dirty_count > 0 || self.missing_count > 0 || self.error_count > 0
    }
    
    /// 合計リポジトリ数
    pub fn total_count(&self) -> usize {
        self.repositories.len()
    }
}

/// ステータス確認のユースケース
pub struct StatusCheckUseCase {
    /// 設定
    config: StatusCheckConfig,
}

impl StatusCheckUseCase {
    /// 新しいStatusCheckUseCaseインスタンスを作成
    pub fn new(config: StatusCheckConfig) -> Self {
        Self { config }
    }

    /// ステータス確認を実行
    pub async fn execute(&self, workspace: &Workspace) -> Result<StatusResult, StatusCheckError> {
        // 1. ワークスペースの初期化チェック
        self.check_workspace_initialized(workspace)?;
        
        // 2. ステータス確認対象リポジトリの決定
        let target_repos = self.determine_target_repositories(workspace)?;
        
        // 3. 各リポジトリのステータス確認
        let mut result = StatusResult::new();
        for repo in target_repos {
            let status = self.check_repository_status(&repo, workspace).await?;
            result.add_repository(status);
        }
        
        Ok(result)
    }
    
    /// ワークスペースが初期化済みかチェック
    fn check_workspace_initialized(&self, workspace: &Workspace) -> Result<(), StatusCheckError> {
        if !workspace.is_initialized() {
            return Err(StatusCheckError::WorkspaceNotInitialized(
                workspace.root_path.display().to_string()
            ));
        }
        Ok(())
    }
    
    /// ステータス確認対象リポジトリの決定
    fn determine_target_repositories(&self, workspace: &Workspace) -> Result<Vec<ManifestRepo>, StatusCheckError> {
        let manifest = workspace.manifest.as_ref().ok_or_else(|| {
            StatusCheckError::GitOperationFailed("Manifest not loaded".to_string())
        })?;
        
        let mut target_repos = Vec::new();
        
        if let Some(groups) = &self.config.groups {
            // 指定されたグループのリポジトリのみ
            for group_name in groups {
                let repos_in_group = manifest.get_repos_in_group(group_name);
                target_repos.extend(repos_in_group.into_iter().cloned());
            }
        } else {
            // 全てのリポジトリ
            target_repos = manifest.repos.clone();
        }
        
        Ok(target_repos)
    }
    
    /// 単一リポジトリのステータス確認
    async fn check_repository_status(
        &self, 
        repo: &ManifestRepo, 
        workspace: &Workspace
    ) -> Result<RepositoryStatus, StatusCheckError> {
        let repo_path = workspace.repo_path(&repo.dest);
        let mut status = RepositoryStatus::new(repo.dest.clone());
        
        if !repo_path.exists() {
            status.state = RepositoryState::Missing;
            return Ok(status);
        }
        
        // Git操作を実行してステータスを取得
        match self.perform_git_status_check(&repo_path, repo).await {
            Ok(git_status) => {
                status = status
                    .with_branch_info(git_status.current_branch, Some(repo.branch.clone().unwrap_or_else(|| "main".to_string())))
                    .with_file_changes(git_status.untracked_files, git_status.modified_files, git_status.staged_files)
                    .with_remote_diff(git_status.commits_ahead, git_status.commits_behind);
            }
            Err(e) => {
                status = status.with_error(e.to_string());
            }
        }
        
        Ok(status)
    }
    
    /// Git status情報を取得
    async fn perform_git_status_check(
        &self, 
        repo_path: &PathBuf, 
        _repo: &ManifestRepo
    ) -> Result<GitStatusInfo, StatusCheckError> {
        if self.config.verbose {
            println!("Checking status for {}", repo_path.display());
        }
        
        let git_repo = GitRepository::open(repo_path)?;
        let repo_status = git_repo.status()?;
        
        Ok(GitStatusInfo {
            current_branch: repo_status.current_branch,
            untracked_files: repo_status.untracked_files.len(),
            modified_files: repo_status.modified_files.len(),
            staged_files: repo_status.staged_files.len(),
            commits_ahead: repo_status.ahead,
            commits_behind: repo_status.behind,
        })
    }
}

/// Git status情報
#[derive(Debug)]
struct GitStatusInfo {
    current_branch: Option<String>,
    untracked_files: usize,
    modified_files: usize,
    staged_files: usize,
    commits_ahead: usize,
    commits_behind: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use crate::domain::entities::workspace::WorkspaceConfig;
    use crate::domain::entities::manifest::Manifest;
    
    #[test]
    fn test_status_config_default() {
        let config = StatusCheckConfig::default();
        assert!(config.groups.is_none());
        assert!(!config.show_branch);
        assert!(!config.compact);
        assert!(!config.verbose);
    }
    
    #[test]
    fn test_repository_status_creation() {
        let status = RepositoryStatus::new("test/repo".to_string());
        assert_eq!(status.dest, "test/repo");
        assert_eq!(status.state, RepositoryState::Missing);
        assert!(status.current_branch.is_none());
        assert!(status.has_issues()); // Missing状態は has_issues = true
    }
    
    #[test]
    fn test_repository_status_has_issues() {
        let clean_status = RepositoryStatus::new("repo".to_string())
            .with_file_changes(0, 0, 0);
        assert_eq!(clean_status.state, RepositoryState::Clean);
        assert!(!clean_status.has_issues());
        
        let dirty_status = RepositoryStatus::new("repo".to_string())
            .with_file_changes(1, 0, 0);
        assert_eq!(dirty_status.state, RepositoryState::Dirty);
        assert!(dirty_status.has_issues());
        
        let error_status = RepositoryStatus::new("repo".to_string())
            .with_error("Git error".to_string());
        assert_eq!(error_status.state, RepositoryState::Error);
        assert!(error_status.has_issues());
    }
    
    #[test]
    fn test_status_result_counting() {
        let mut result = StatusResult::new();
        
        let clean_status = RepositoryStatus::new("clean".to_string())
            .with_file_changes(0, 0, 0);
        let dirty_status = RepositoryStatus::new("dirty".to_string())
            .with_file_changes(1, 0, 0);
        let missing_status = RepositoryStatus::new("missing".to_string()); // デフォルトでMissing
        
        result.add_repository(clean_status);
        result.add_repository(dirty_status);
        result.add_repository(missing_status);
        
        assert_eq!(result.clean_count, 1);
        assert_eq!(result.dirty_count, 1);
        assert_eq!(result.missing_count, 1);
        assert_eq!(result.error_count, 0);
        assert_eq!(result.total_count(), 3);
        assert!(result.has_issues());
    }
    
    #[tokio::test]
    async fn test_workspace_initialization_check() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_config = WorkspaceConfig::new("https://example.com/manifest.git", "main");
        let workspace = Workspace::new(temp_dir.path().to_path_buf(), workspace_config);
        
        let config = StatusCheckConfig::default();
        let use_case = StatusCheckUseCase::new(config);
        
        // 未初期化の場合はエラー
        let result = use_case.check_workspace_initialized(&workspace);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_target_repositories_determination() {
        let manifest = Manifest::new(vec![]);
        let temp_dir = TempDir::new().unwrap();
        let workspace_config = WorkspaceConfig::new("https://example.com/manifest.git", "main");
        let workspace = Workspace::new(temp_dir.path().to_path_buf(), workspace_config)
            .with_manifest(manifest);
        
        let config = StatusCheckConfig::default();
        let use_case = StatusCheckUseCase::new(config);
        
        let result = use_case.determine_target_repositories(&workspace);
        assert!(result.is_ok());
    }
}