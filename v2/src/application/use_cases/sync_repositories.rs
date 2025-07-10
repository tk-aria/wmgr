use std::path::PathBuf;
use thiserror::Error;
use crate::domain::entities::{
    workspace::{Workspace, WorkspaceStatus}, 
    manifest::ManifestRepo
};
use crate::domain::value_objects::branch_name::BranchName;

/// SyncRepositories関連のエラー
#[derive(Debug, Error)]
pub enum SyncRepositoriesError {
    #[error("Workspace not initialized: {0}")]
    WorkspaceNotInitialized(String),
    
    #[error("Manifest update failed: {0}")]
    ManifestUpdateFailed(String),
    
    #[error("Repository clone failed: {0}")]
    RepositoryCloneFailed(String),
    
    #[error("Remote update failed for repo '{repo}': {error}")]
    RemoteUpdateFailed { repo: String, error: String },
    
    #[error("Branch sync failed for repo '{repo}': {error}")]
    BranchSyncFailed { repo: String, error: String },
    
    #[error("Git operation failed: {0}")]
    GitOperationFailed(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Git URL error: {0}")]
    GitUrlError(#[from] crate::domain::value_objects::git_url::GitUrlError),
    
    #[error("Branch name error: {0}")]
    BranchNameError(#[from] crate::domain::value_objects::branch_name::BranchNameError),
    
    #[error("File path error: {0}")]
    FilePathError(#[from] crate::domain::value_objects::file_path::FilePathError),
}

/// リポジトリ同期の設定
#[derive(Debug, Clone)]
pub struct SyncRepositoriesConfig {
    /// 特定のグループのみを同期するか（Noneの場合は全て）
    pub groups: Option<Vec<String>>,
    
    /// 強制的に同期するか（ローカル変更を無視）
    pub force: bool,
    
    /// 正しいブランチへの切り替えを無効にするか
    pub no_correct_branch: bool,
    
    /// 並列実行の最大数（Noneの場合はCPU数）
    pub parallel_jobs: Option<usize>,
    
    /// 詳細ログを出力するか
    pub verbose: bool,
}

impl Default for SyncRepositoriesConfig {
    fn default() -> Self {
        Self {
            groups: None,
            force: false,
            no_correct_branch: false,
            parallel_jobs: None,
            verbose: false,
        }
    }
}

/// 同期操作の結果
#[derive(Debug, Clone)]
pub struct SyncResult {
    /// 同期されたリポジトリの数
    pub synced_count: usize,
    
    /// 新規クローンされたリポジトリの数
    pub cloned_count: usize,
    
    /// 更新されたリポジトリの数
    pub updated_count: usize,
    
    /// スキップされたリポジトリの数（エラーや設定により）
    pub skipped_count: usize,
    
    /// 発生したエラーのリスト
    pub errors: Vec<String>,
}

impl SyncResult {
    pub fn new() -> Self {
        Self {
            synced_count: 0,
            cloned_count: 0,
            updated_count: 0,
            skipped_count: 0,
            errors: Vec::new(),
        }
    }
    
    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
    }
    
    pub fn is_success(&self) -> bool {
        self.errors.is_empty()
    }
}

/// リポジトリ同期のユースケース
pub struct SyncRepositoriesUseCase {
    /// 同期設定
    config: SyncRepositoriesConfig,
}

impl SyncRepositoriesUseCase {
    /// 新しいSyncRepositoriesUseCaseインスタンスを作成
    pub fn new(config: SyncRepositoriesConfig) -> Self {
        Self { config }
    }

    /// リポジトリ同期を実行
    pub async fn execute(&self, workspace: &mut Workspace) -> Result<SyncResult, SyncRepositoriesError> {
        // 1. ワークスペースの初期化チェック
        self.check_workspace_initialized(workspace)?;
        
        // 2. マニフェストの更新
        self.update_manifest(workspace).await?;
        
        // 3. 同期対象リポジトリの決定
        let target_repos = self.determine_target_repositories(workspace)?;
        
        // 4. リポジトリの同期実行
        let mut result = SyncResult::new();
        self.sync_repositories(&target_repos, workspace, &mut result).await?;
        
        // 5. ワークスペース状態の更新
        workspace.status = WorkspaceStatus::Initialized;
        
        Ok(result)
    }
    
    /// ワークスペースが初期化済みかチェック
    fn check_workspace_initialized(&self, workspace: &Workspace) -> Result<(), SyncRepositoriesError> {
        if !workspace.is_initialized() {
            return Err(SyncRepositoriesError::WorkspaceNotInitialized(
                workspace.root_path.display().to_string()
            ));
        }
        Ok(())
    }
    
    /// マニフェストの更新
    async fn update_manifest(&self, workspace: &mut Workspace) -> Result<(), SyncRepositoriesError> {
        // ローカルファーストアプローチ: マニフェストファイルの再読み込みのみ
        let manifest_file = workspace.manifest_file_path();
        
        if !manifest_file.exists() {
            return Err(SyncRepositoriesError::ManifestUpdateFailed(
                format!("Manifest file not found: {}", manifest_file.display())
            ));
        }
        
        if self.config.verbose {
            println!("Reloading manifest from: {}", manifest_file.display());
        }
        
        // ManifestStoreを使ってマニフェストファイルを再読み込み
        let manifest = self.reload_manifest_from_file(&manifest_file).await?;
        workspace.manifest = Some(manifest);
        
        if self.config.verbose {
            println!("Manifest reloaded successfully");
        }
        
        Ok(())
    }
    
    /// マニフェストファイルから再読み込み（ローカルファーストアプローチ）
    async fn reload_manifest_from_file(&self, manifest_file: &std::path::Path) -> Result<crate::domain::entities::manifest::Manifest, SyncRepositoriesError> {
        use crate::infrastructure::filesystem::manifest_store::{ManifestStore, ManifestStoreError};
        
        let mut manifest_store = ManifestStore::new();
        let processed_manifest = manifest_store.read_manifest(manifest_file).await
            .map_err(|e| match e {
                ManifestStoreError::ManifestFileNotFound(path) => {
                    SyncRepositoriesError::ManifestUpdateFailed(format!("Manifest file not found: {}", path))
                }
                ManifestStoreError::YamlParsingFailed(err) => {
                    SyncRepositoriesError::ManifestUpdateFailed(format!("YAML parsing failed: {}", err))
                }
                _ => SyncRepositoriesError::ManifestUpdateFailed(format!("Failed to read manifest: {}", e))
            })?;
        
        Ok(processed_manifest.manifest)
    }
    
    
    /// 同期対象リポジトリの決定
    fn determine_target_repositories(&self, workspace: &Workspace) -> Result<Vec<ManifestRepo>, SyncRepositoriesError> {
        let manifest = workspace.manifest.as_ref().ok_or_else(|| {
            SyncRepositoriesError::ManifestUpdateFailed("Manifest not loaded".to_string())
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
    
    /// リポジトリの同期実行
    async fn sync_repositories(
        &self, 
        target_repos: &[ManifestRepo], 
        workspace: &Workspace, 
        result: &mut SyncResult
    ) -> Result<(), SyncRepositoriesError> {
        for repo in target_repos {
            match self.sync_single_repository(repo, workspace).await {
                Ok(operation) => {
                    match operation {
                        SyncOperation::Cloned => result.cloned_count += 1,
                        SyncOperation::Updated => result.updated_count += 1,
                        SyncOperation::Skipped => result.skipped_count += 1,
                    }
                    result.synced_count += 1;
                }
                Err(e) => {
                    result.add_error(format!("Failed to sync {}: {}", repo.dest, e));
                    result.skipped_count += 1;
                }
            }
        }
        
        Ok(())
    }
    
    /// 単一リポジトリの同期
    async fn sync_single_repository(
        &self, 
        repo: &ManifestRepo, 
        workspace: &Workspace
    ) -> Result<SyncOperation, SyncRepositoriesError> {
        let repo_path = workspace.repo_path(&repo.dest);
        
        if !repo_path.exists() {
            // リポジトリが存在しない場合はクローン
            self.clone_repository(repo, &repo_path).await?;
            Ok(SyncOperation::Cloned)
        } else {
            // 既存リポジトリの更新
            self.update_repository(repo, &repo_path).await?;
            Ok(SyncOperation::Updated)
        }
    }
    
    /// リポジトリのクローン
    async fn clone_repository(&self, repo: &ManifestRepo, target_path: &PathBuf) -> Result<(), SyncRepositoriesError> {
        if self.config.verbose {
            println!("Cloning {} to {}", repo.url, target_path.display());
        }
        
        // ディレクトリの親を作成
        if let Some(parent) = target_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        // Git clone実行（疑似実装）
        self.perform_git_clone(&repo.url, target_path, repo.branch.as_deref()).await?;
        
        Ok(())
    }
    
    /// Git clone実行（実際のGit操作）
    async fn perform_git_clone(
        &self, 
        url: &str, 
        target_path: &PathBuf, 
        branch: Option<&str>
    ) -> Result<(), SyncRepositoriesError> {
        use crate::infrastructure::git::repository::{GitRepository, CloneConfig};
        use crate::domain::value_objects::{git_url::GitUrl, file_path::FilePath};
        
        if self.config.verbose {
            println!("Starting Git clone: {} -> {}", url, target_path.display());
        }
        
        // URLとパスの検証・変換
        let git_url = GitUrl::new(url)?;
        let file_path = FilePath::new(target_path.to_string_lossy().as_ref())?;
        
        // クローン設定
        let clone_config = CloneConfig {
            branch: branch.map(|b| b.to_string()),
            shallow: false,
            depth: None,
            recursive: false,
            progress_callback: None,
        };
        
        // Git クローン実行
        let _repo = GitRepository::clone(&git_url, &file_path, clone_config).await
            .map_err(|e| SyncRepositoriesError::RepositoryCloneFailed(
                format!("Failed to clone {}: {}", url, e)
            ))?;
        
        if self.config.verbose {
            println!("Successfully cloned: {} -> {}", url, target_path.display());
        }
        
        Ok(())
    }
    
    /// 既存リポジトリの更新
    async fn update_repository(&self, repo: &ManifestRepo, repo_path: &PathBuf) -> Result<(), SyncRepositoriesError> {
        if self.config.verbose {
            println!("Updating repository at {}", repo_path.display());
        }
        
        // 1. リモートURLの更新
        self.update_remotes(repo, repo_path).await?;
        
        // 2. フェッチ実行
        self.perform_git_fetch(repo_path).await?;
        
        // 3. ブランチの同期
        if !self.config.no_correct_branch {
            self.sync_branch(repo, repo_path).await?;
        }
        
        Ok(())
    }
    
    /// リモート設定の更新
    async fn update_remotes(&self, repo: &ManifestRepo, repo_path: &PathBuf) -> Result<(), SyncRepositoriesError> {
        use crate::infrastructure::git::repository::GitRepository;
        use crate::infrastructure::git::remote::GitRemoteManager;
        use crate::domain::value_objects::git_url::GitUrl;
        
        if self.config.verbose {
            println!("Updating remotes for {}", repo_path.display());
        }
        
        // 既存リポジトリを開く
        let git_repo = GitRepository::open(repo_path)
            .map_err(|e| SyncRepositoriesError::GitOperationFailed(
                format!("Failed to open repository at {}: {}", repo_path.display(), e)
            ))?;
        
        // リモート管理オブジェクトを作成
        let remote_manager = GitRemoteManager::new(git_repo.git2_repo());
        
        // URLを検証・変換
        let git_url = GitUrl::new(&repo.url)?;
        
        // originリモートのURL更新
        if remote_manager.remote_exists("origin") {
            remote_manager.set_remote_url("origin", &git_url)
                .map_err(|e| SyncRepositoriesError::RemoteUpdateFailed {
                    repo: repo.dest.clone(),
                    error: format!("Failed to update origin remote URL: {}", e),
                })?;
            
            if self.config.verbose {
                println!("Updated origin remote URL to: {}", repo.url);
            }
        } else {
            // originリモートが存在しない場合は追加
            remote_manager.add_remote("origin", &git_url)
                .map_err(|e| SyncRepositoriesError::RemoteUpdateFailed {
                    repo: repo.dest.clone(),
                    error: format!("Failed to add origin remote: {}", e),
                })?;
            
            if self.config.verbose {
                println!("Added origin remote with URL: {}", repo.url);
            }
        }
        
        Ok(())
    }
    
    
    /// Git fetchの実行
    async fn perform_git_fetch(&self, _repo_path: &PathBuf) -> Result<(), SyncRepositoriesError> {
        // TODO: 実際のGit操作はインフラストラクチャ層で実装
        if self.config.verbose {
            println!("Fetching latest changes...");
        }
        Ok(())
    }
    
    /// ブランチの同期（fast-forward merge）
    async fn sync_branch(&self, repo: &ManifestRepo, repo_path: &PathBuf) -> Result<(), SyncRepositoriesError> {
        let target_branch = repo.branch.as_deref().unwrap_or("main");
        
        if self.config.verbose {
            println!("Syncing branch '{}' in {}", target_branch, repo_path.display());
        }
        
        // ブランチ名の検証
        let _branch_name = BranchName::new(target_branch)?;
        
        // 1. 現在のブランチをチェック
        let current_branch = self.get_current_branch(repo_path).await?;
        
        // 2. 必要に応じてブランチを切り替え
        if current_branch != target_branch {
            self.perform_git_checkout(repo_path, target_branch).await?;
        }
        
        // 3. Fast-forward merge実行
        if !self.config.force {
            // ローカル変更がある場合は警告
            let has_local_changes = self.check_local_changes(repo_path).await?;
            if has_local_changes {
                return Err(SyncRepositoriesError::BranchSyncFailed {
                    repo: repo.dest.clone(),
                    error: "Local changes detected. Use --force to override.".to_string(),
                });
            }
        }
        
        self.perform_git_merge_ff(repo_path, target_branch).await?;
        
        Ok(())
    }
    
    /// 現在のブランチを取得（疑似実装）
    async fn get_current_branch(&self, _repo_path: &PathBuf) -> Result<String, SyncRepositoriesError> {
        // TODO: 実際のGit操作はインフラストラクチャ層で実装
        Ok("main".to_string())
    }
    
    /// Git checkoutの実行（疑似実装）
    async fn perform_git_checkout(&self, _repo_path: &PathBuf, _branch: &str) -> Result<(), SyncRepositoriesError> {
        // TODO: 実際のGit操作はインフラストラクチャ層で実装
        Ok(())
    }
    
    /// ローカル変更の有無をチェック（疑似実装）
    async fn check_local_changes(&self, _repo_path: &PathBuf) -> Result<bool, SyncRepositoriesError> {
        // TODO: 実際のGit操作はインフラストラクチャ層で実装
        Ok(false)
    }
    
    /// Fast-forward mergeの実行（疑似実装）
    async fn perform_git_merge_ff(&self, _repo_path: &PathBuf, _branch: &str) -> Result<(), SyncRepositoriesError> {
        // TODO: 実際のGit操作はインフラストラクチャ層で実装
        Ok(())
    }
}

/// 同期操作の種類
#[derive(Debug, Clone, PartialEq, Eq)]
enum SyncOperation {
    /// 新規クローン
    Cloned,
    /// 既存リポジトリの更新
    Updated,
    /// スキップ（エラーまたは設定による）
    Skipped,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use crate::domain::entities::workspace::WorkspaceConfig;
    
    #[test]
    fn test_sync_config_default() {
        let config = SyncRepositoriesConfig::default();
        assert!(config.groups.is_none());
        assert!(!config.force);
        assert!(!config.no_correct_branch);
        assert!(config.parallel_jobs.is_none());
        assert!(!config.verbose);
    }
    
    #[test]
    fn test_sync_result_creation() {
        let mut result = SyncResult::new();
        assert_eq!(result.synced_count, 0);
        assert_eq!(result.cloned_count, 0);
        assert_eq!(result.updated_count, 0);
        assert_eq!(result.skipped_count, 0);
        assert!(result.is_success());
        
        result.add_error("Test error".to_string());
        assert!(!result.is_success());
        assert_eq!(result.errors.len(), 1);
    }
    
    #[tokio::test]
    async fn test_workspace_initialization_check() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_config = WorkspaceConfig::new("https://example.com/manifest.git", "main");
        let mut workspace = Workspace::new(temp_dir.path().to_path_buf(), workspace_config);
        
        let config = SyncRepositoriesConfig::default();
        let use_case = SyncRepositoriesUseCase::new(config);
        
        // 未初期化の場合はエラー
        let result = use_case.check_workspace_initialized(&workspace);
        assert!(result.is_err());
        
        // 初期化済みに設定
        workspace.status = WorkspaceStatus::Initialized;
        let result = use_case.check_workspace_initialized(&workspace);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_target_repositories_determination() {
        let manifest = Manifest::new(vec![]);
        let temp_dir = TempDir::new().unwrap();
        let workspace_config = WorkspaceConfig::new("https://example.com/manifest.git", "main");
        let workspace = Workspace::new(temp_dir.path().to_path_buf(), workspace_config)
            .with_manifest(manifest);
        
        let config = SyncRepositoriesConfig::default();
        let use_case = SyncRepositoriesUseCase::new(config);
        
        let result = use_case.determine_target_repositories(&workspace);
        assert!(result.is_ok());
    }
}