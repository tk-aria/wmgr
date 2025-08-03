use crate::domain::entities::{
    manifest::ManifestRepo,
    workspace::{Workspace, WorkspaceStatus},
};
use crate::domain::value_objects::branch_name::BranchName;
use crate::infrastructure::scm::{ScmFactory, ScmError};
use std::path::PathBuf;
use thiserror::Error;

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

    #[error("SCM operation failed: {0}")]
    ScmOperationFailed(#[from] ScmError),

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

    /// 子ディレクトリのワークスペースも再帰的に同期するか
    pub recursive: bool,
}

impl Default for SyncRepositoriesConfig {
    fn default() -> Self {
        Self {
            groups: None,
            force: false,
            no_correct_branch: false,
            parallel_jobs: None,
            verbose: false,
            recursive: true,
        }
    }
}

impl SyncRepositoriesConfig {
    pub fn with_groups(mut self, groups: Vec<String>) -> Self {
        self.groups = Some(groups);
        self
    }

    pub fn with_force(mut self, force: bool) -> Self {
        self.force = force;
        self
    }

    pub fn with_no_correct_branch(mut self, no_correct_branch: bool) -> Self {
        self.no_correct_branch = no_correct_branch;
        self
    }

    pub fn with_parallel_jobs(mut self, parallel_jobs: usize) -> Self {
        self.parallel_jobs = Some(parallel_jobs);
        self
    }

    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    pub fn with_recursive(mut self, recursive: bool) -> Self {
        self.recursive = recursive;
        self
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

    pub fn total_count(&self) -> usize {
        self.cloned_count + self.updated_count + self.skipped_count
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
    pub async fn execute(
        &self,
        workspace: &mut Workspace,
    ) -> Result<SyncResult, SyncRepositoriesError> {
        // 1. ワークスペースの初期化チェック
        self.check_workspace_initialized(workspace)?;

        // 2. マニフェストの更新
        self.update_manifest(workspace).await?;

        // 3. 同期対象リポジトリの決定
        let target_repos = self.determine_target_repositories(workspace)?;

        // 4. リポジトリの同期実行
        let mut result = SyncResult::new();
        self.sync_repositories(&target_repos, workspace, &mut result)
            .await?;

        // 5. 再帰的な子ワークスペースの同期（recursive フラグが有効な場合）
        if self.config.recursive {
            self.sync_child_workspaces(workspace, &mut result).await?;
        }

        // 6. ワークスペース状態の更新
        workspace.status = WorkspaceStatus::Initialized;

        Ok(result)
    }

    /// ワークスペースが初期化済みかチェック
    fn check_workspace_initialized(
        &self,
        workspace: &Workspace,
    ) -> Result<(), SyncRepositoriesError> {
        if !workspace.is_initialized() {
            return Err(SyncRepositoriesError::WorkspaceNotInitialized(
                workspace.root_path.display().to_string(),
            ));
        }
        Ok(())
    }

    /// マニフェストの更新
    async fn update_manifest(
        &self,
        workspace: &mut Workspace,
    ) -> Result<(), SyncRepositoriesError> {
        // ローカルファーストアプローチ: マニフェストファイルの再読み込みのみ
        let manifest_file = workspace.manifest_file_path();

        if !manifest_file.exists() {
            return Err(SyncRepositoriesError::ManifestUpdateFailed(format!(
                "Manifest file not found: {}",
                manifest_file.display()
            )));
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
    async fn reload_manifest_from_file(
        &self,
        manifest_file: &std::path::Path,
    ) -> Result<crate::domain::entities::manifest::Manifest, SyncRepositoriesError> {
        use crate::infrastructure::filesystem::manifest_store::{
            ManifestStore, ManifestStoreError,
        };

        let mut manifest_store = ManifestStore::new();
        let processed_manifest =
            manifest_store
                .read_manifest(manifest_file)
                .await
                .map_err(|e| match e {
                    ManifestStoreError::ManifestFileNotFound(path) => {
                        SyncRepositoriesError::ManifestUpdateFailed(format!(
                            "Manifest file not found: {}",
                            path
                        ))
                    }
                    ManifestStoreError::YamlParsingFailed(err) => {
                        SyncRepositoriesError::ManifestUpdateFailed(format!(
                            "YAML parsing failed: {}",
                            err
                        ))
                    }
                    _ => SyncRepositoriesError::ManifestUpdateFailed(format!(
                        "Failed to read manifest: {}",
                        e
                    )),
                })?;

        Ok(processed_manifest.manifest)
    }

    /// 同期対象リポジトリの決定
    fn determine_target_repositories(
        &self,
        workspace: &Workspace,
    ) -> Result<Vec<ManifestRepo>, SyncRepositoriesError> {
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
        result: &mut SyncResult,
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
        workspace: &Workspace,
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

    /// リポジトリのクローン（SCM対応）
    async fn clone_repository(
        &self,
        repo: &ManifestRepo,
        target_path: &PathBuf,
    ) -> Result<(), SyncRepositoriesError> {
        if self.config.verbose {
            println!("Cloning {} ({}) to {}", repo.url, repo.scm, target_path.display());
        }

        // ディレクトリの親を作成
        if let Some(parent) = target_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // SCM操作の実行
        self.perform_scm_clone(repo, target_path).await?;

        Ok(())
    }

    /// SCMクローン実行（マルチSCM対応）
    async fn perform_scm_clone(
        &self,
        repo: &ManifestRepo,
        target_path: &PathBuf,
    ) -> Result<(), SyncRepositoriesError> {
        if self.config.verbose {
            println!("Starting {} clone: {} -> {}", repo.scm, repo.url, target_path.display());
        }

        // SCM操作インスタンスを作成
        let scm = ScmFactory::create_scm(repo.scm.clone())?;
        
        // クローンオプションを構築
        let clone_options = repo.to_clone_options();

        // SCMクローンを実行
        scm.clone_repository(&repo.url, target_path, &clone_options)
            .await
            .map_err(|e| {
                SyncRepositoriesError::RepositoryCloneFailed(format!(
                    "Failed to clone {} ({}): {}",
                    repo.url, repo.scm, e
                ))
            })?;

        if self.config.verbose {
            println!("Successfully cloned: {} -> {}", repo.url, target_path.display());
        }

        Ok(())
    }

    /// Git clone実行（実際のGit操作）（レガシー - 後で削除予定）
    async fn perform_git_clone(
        &self,
        url: &str,
        target_path: &PathBuf,
        branch: Option<&str>,
    ) -> Result<(), SyncRepositoriesError> {
        use crate::domain::value_objects::{file_path::FilePath, git_url::GitUrl};
        use crate::infrastructure::git::repository::{CloneConfig, GitRepository};

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
        let _repo = GitRepository::clone(&git_url, &file_path, clone_config)
            .await
            .map_err(|e| {
                SyncRepositoriesError::RepositoryCloneFailed(format!(
                    "Failed to clone {}: {}",
                    url, e
                ))
            })?;

        if self.config.verbose {
            println!("Successfully cloned: {} -> {}", url, target_path.display());
        }

        Ok(())
    }

    /// 既存リポジトリの更新（SCM対応）
    async fn update_repository(
        &self,
        repo: &ManifestRepo,
        repo_path: &PathBuf,
    ) -> Result<(), SyncRepositoriesError> {
        if self.config.verbose {
            println!("Updating {} repository at {}", repo.scm, repo_path.display());
        }

        // SCM操作インスタンスを作成
        let scm = ScmFactory::create_scm(repo.scm.clone())?;

        // リポジトリの種別を確認
        if !scm.is_repository(repo_path) {
            return Err(SyncRepositoriesError::GitOperationFailed(format!(
                "Path {} is not a valid {} repository",
                repo_path.display(), repo.scm
            )));
        }

        // 同期オプションを構築
        let sync_options = repo.to_sync_options(self.config.force);

        // SCM同期を実行
        scm.sync_repository(repo_path, &sync_options)
            .await
            .map_err(|e| {
                SyncRepositoriesError::RemoteUpdateFailed {
                    repo: repo.dest.clone(),
                    error: format!("Failed to sync {} repository: {}", repo.scm, e),
                }
            })?;

        if self.config.verbose {
            println!("Successfully updated {} repository at {}", repo.scm, repo_path.display());
        }

        Ok(())
    }

    /// リモート設定の更新
    async fn update_remotes(
        &self,
        repo: &ManifestRepo,
        repo_path: &PathBuf,
    ) -> Result<(), SyncRepositoriesError> {
        use crate::domain::value_objects::git_url::GitUrl;
        use crate::infrastructure::git::remote::GitRemoteManager;
        use crate::infrastructure::git::repository::GitRepository;

        if self.config.verbose {
            println!("Updating remotes for {}", repo_path.display());
        }

        // 既存リポジトリを開く
        let git_repo = GitRepository::open(repo_path).map_err(|e| {
            SyncRepositoriesError::GitOperationFailed(format!(
                "Failed to open repository at {}: {}",
                repo_path.display(),
                e
            ))
        })?;

        // リモート管理オブジェクトを作成
        let remote_manager = GitRemoteManager::new(git_repo.git2_repo());

        // URLを検証・変換
        let git_url = GitUrl::new(&repo.url)?;

        // originリモートのURL更新
        if remote_manager.remote_exists("origin") {
            remote_manager
                .set_remote_url("origin", &git_url)
                .map_err(|e| SyncRepositoriesError::RemoteUpdateFailed {
                    repo: repo.dest.clone(),
                    error: format!("Failed to update origin remote URL: {}", e),
                })?;

            if self.config.verbose {
                println!("Updated origin remote URL to: {}", repo.url);
            }
        } else {
            // originリモートが存在しない場合は追加
            remote_manager.add_remote("origin", &git_url).map_err(|e| {
                SyncRepositoriesError::RemoteUpdateFailed {
                    repo: repo.dest.clone(),
                    error: format!("Failed to add origin remote: {}", e),
                }
            })?;

            if self.config.verbose {
                println!("Added origin remote with URL: {}", repo.url);
            }
        }

        Ok(())
    }

    /// Git fetchの実行
    async fn perform_git_fetch(&self, repo_path: &PathBuf) -> Result<(), SyncRepositoriesError> {
        use crate::infrastructure::git::repository::{FetchConfig, GitRepository};

        if self.config.verbose {
            println!("Fetching latest changes from origin...");
        }

        // 既存リポジトリを開く
        let git_repo = GitRepository::open(repo_path).map_err(|e| {
            SyncRepositoriesError::GitOperationFailed(format!(
                "Failed to open repository at {}: {}",
                repo_path.display(),
                e
            ))
        })?;

        // フェッチ設定
        let fetch_config = FetchConfig {
            remote_name: "origin".to_string(),
            refs: None, // すべてのリファレンスをフェッチ
            progress_callback: None,
        };

        // Git fetch実行
        git_repo.fetch(fetch_config).await.map_err(|e| {
            SyncRepositoriesError::GitOperationFailed(format!("Failed to fetch from origin: {}", e))
        })?;

        if self.config.verbose {
            println!("Successfully fetched latest changes");
        }

        Ok(())
    }

    /// ブランチの同期（fast-forward merge）
    async fn sync_branch(
        &self,
        repo: &ManifestRepo,
        repo_path: &PathBuf,
    ) -> Result<(), SyncRepositoriesError> {
        let target_branch = repo.branch.as_deref().unwrap_or("main");

        if self.config.verbose {
            println!(
                "Syncing branch '{}' in {}",
                target_branch,
                repo_path.display()
            );
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

    /// 現在のブランチを取得
    async fn get_current_branch(
        &self,
        repo_path: &PathBuf,
    ) -> Result<String, SyncRepositoriesError> {
        use crate::infrastructure::git::repository::GitRepository;

        // 既存リポジトリを開く
        let git_repo = GitRepository::open(repo_path).map_err(|e| {
            SyncRepositoriesError::GitOperationFailed(format!(
                "Failed to open repository at {}: {}",
                repo_path.display(),
                e
            ))
        })?;

        // 現在のブランチを取得
        git_repo.get_current_branch().map_err(|e| {
            SyncRepositoriesError::GitOperationFailed(format!(
                "Failed to get current branch: {}",
                e
            ))
        })
    }

    /// Git checkoutの実行
    async fn perform_git_checkout(
        &self,
        repo_path: &PathBuf,
        branch: &str,
    ) -> Result<(), SyncRepositoriesError> {
        use crate::infrastructure::git::repository::GitRepository;

        if self.config.verbose {
            println!(
                "Checking out branch '{}' in {}",
                branch,
                repo_path.display()
            );
        }

        // 既存リポジトリを開く
        let git_repo = GitRepository::open(repo_path).map_err(|e| {
            SyncRepositoriesError::GitOperationFailed(format!(
                "Failed to open repository at {}: {}",
                repo_path.display(),
                e
            ))
        })?;

        // ブランチチェックアウト実行
        git_repo
            .checkout(branch)
            .map_err(|e| SyncRepositoriesError::BranchSyncFailed {
                repo: repo_path.display().to_string(),
                error: format!("Failed to checkout branch '{}': {}", branch, e),
            })?;

        if self.config.verbose {
            println!("Successfully checked out branch '{}'", branch);
        }

        Ok(())
    }

    /// ローカル変更の有無をチェック
    async fn check_local_changes(
        &self,
        repo_path: &PathBuf,
    ) -> Result<bool, SyncRepositoriesError> {
        use crate::infrastructure::git::repository::GitRepository;

        // 既存リポジトリを開く
        let git_repo = GitRepository::open(repo_path).map_err(|e| {
            SyncRepositoriesError::GitOperationFailed(format!(
                "Failed to open repository at {}: {}",
                repo_path.display(),
                e
            ))
        })?;

        // ワーキングディレクトリのクリーン状態をチェック
        let is_clean = git_repo.is_working_directory_clean().map_err(|e| {
            SyncRepositoriesError::GitOperationFailed(format!(
                "Failed to check working directory status: {}",
                e
            ))
        })?;

        // クリーンでない場合は変更があることを示す
        Ok(!is_clean)
    }

    /// Fast-forward mergeの実行
    async fn perform_git_merge_ff(
        &self,
        repo_path: &PathBuf,
        branch: &str,
    ) -> Result<(), SyncRepositoriesError> {
        use crate::infrastructure::git::repository::GitRepository;

        if self.config.verbose {
            println!(
                "Performing fast-forward merge for branch '{}' in {}",
                branch,
                repo_path.display()
            );
        }

        // 既存リポジトリを開く
        let git_repo = GitRepository::open(repo_path).map_err(|e| {
            SyncRepositoriesError::GitOperationFailed(format!(
                "Failed to open repository at {}: {}",
                repo_path.display(),
                e
            ))
        })?;

        // Fast-forward merge実行
        git_repo.fast_forward_merge(branch).map_err(|e| {
            SyncRepositoriesError::BranchSyncFailed {
                repo: repo_path.display().to_string(),
                error: format!("Failed to fast-forward merge branch '{}': {}", branch, e),
            }
        })?;

        if self.config.verbose {
            println!(
                "Successfully performed fast-forward merge for branch '{}'",
                branch
            );
        }

        Ok(())
    }

    /// 子ディレクトリのワークスペースを再帰的に同期
    async fn sync_child_workspaces(
        &self,
        workspace: &Workspace,
        result: &mut SyncResult,
    ) -> Result<(), SyncRepositoriesError> {
        if self.config.verbose {
            println!("Searching for child workspaces...");
        }

        // 現在のワークスペースのマニフェストを取得
        let manifest = workspace.manifest.as_ref().ok_or_else(|| {
            SyncRepositoriesError::ManifestUpdateFailed("Manifest not loaded".to_string())
        })?;

        // 各リポジトリディレクトリで子ワークスペースを検索
        for repo in &manifest.repos {
            let repo_path = workspace.root_path.join(&repo.dest);
            
            if !repo_path.exists() {
                continue; // リポジトリがまだクローンされていない場合はスキップ
            }

            // 子ディレクトリでワークスペースルートを検索
            if let Some(child_workspace_root) = Workspace::discover_workspace_root(&repo_path) {
                // 親ワークスペースと同じ場合はスキップ（無限ループ防止）
                if child_workspace_root == workspace.root_path {
                    continue;
                }

                if self.config.verbose {
                    println!("Found child workspace at: {}", child_workspace_root.display());
                }

                // 子ワークスペースの同期を実行
                match self.sync_child_workspace(&child_workspace_root, result).await {
                    Ok(_) => {
                        if self.config.verbose {
                            println!("Successfully synced child workspace: {}", child_workspace_root.display());
                        }
                    }
                    Err(e) => {
                        let error_msg = format!(
                            "Failed to sync child workspace {}: {}",
                            child_workspace_root.display(),
                            e
                        );
                        if self.config.verbose {
                            println!("Error: {}", error_msg);
                        }
                        result.add_error(error_msg);
                    }
                }
            }
        }

        Ok(())
    }

    /// 個別の子ワークスペースを同期
    fn sync_child_workspace<'a>(
        &'a self,
        child_workspace_root: &'a std::path::Path,
        result: &'a mut SyncResult,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), SyncRepositoriesError>> + 'a>> {
        Box::pin(async move {
        use crate::domain::entities::workspace::{WorkspaceConfig, WorkspaceStatus};
        use crate::infrastructure::filesystem::manifest_store::ManifestStore;

        // 子ワークスペースを作成
        let child_workspace_config = WorkspaceConfig::default_local();
        let child_workspace = Workspace::new(child_workspace_root.to_path_buf(), child_workspace_config);
        let manifest_file = child_workspace.manifest_file_path();

        // マニフェストファイルが存在するかチェック
        if !manifest_file.exists() {
            return Err(SyncRepositoriesError::ManifestUpdateFailed(format!(
                "Child workspace manifest file not found: {}",
                manifest_file.display()
            )));
        }

        // マニフェストを読み込み
        let mut manifest_store = ManifestStore::new();
        let processed_manifest = manifest_store
            .read_manifest(&manifest_file)
            .await
            .map_err(|e| {
                SyncRepositoriesError::ManifestUpdateFailed(format!(
                    "Failed to load child workspace manifest: {}",
                    e
                ))
            })?;

        // 子ワークスペースの設定
        let mut child_workspace = child_workspace
            .with_status(WorkspaceStatus::Initialized)
            .with_manifest(processed_manifest.manifest);

        // 子ワークスペース用の設定を作成（recursive=falseで無限ループを防止）
        let child_config = SyncRepositoriesConfig {
            groups: self.config.groups.clone(),
            force: self.config.force,
            no_correct_branch: self.config.no_correct_branch,
            parallel_jobs: self.config.parallel_jobs,
            verbose: self.config.verbose,
            recursive: false, // 子ワークスペースでは再帰を無効化
        };

        // 子ワークスペースの同期実行
        let child_use_case = SyncRepositoriesUseCase::new(child_config);
        let child_result = child_use_case.execute(&mut child_workspace).await?;

        // 結果をマージ
        result.synced_count += child_result.synced_count;
        result.cloned_count += child_result.cloned_count;
        result.updated_count += child_result.updated_count;
        result.skipped_count += child_result.skipped_count;
        result.errors.extend(child_result.errors);

        Ok(())
        })
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
    use crate::domain::entities::manifest::Manifest;
    use crate::domain::entities::workspace::WorkspaceConfig;
    use tempfile::TempDir;

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
        let workspace =
            Workspace::new(temp_dir.path().to_path_buf(), workspace_config).with_manifest(manifest);

        let config = SyncRepositoriesConfig::default();
        let use_case = SyncRepositoriesUseCase::new(config);

        let result = use_case.determine_target_repositories(&workspace);
        assert!(result.is_ok());
    }

    #[test]
    fn test_sync_config_with_options() {
        let config = SyncRepositoriesConfig::default()
            .with_groups(vec!["group1".to_string(), "group2".to_string()])
            .with_force(true)
            .with_no_correct_branch(true)
            .with_parallel_jobs(4)
            .with_verbose(true);

        assert_eq!(
            config.groups,
            Some(vec!["group1".to_string(), "group2".to_string()])
        );
        assert!(config.force);
        assert!(config.no_correct_branch);
        assert_eq!(config.parallel_jobs, Some(4));
        assert!(config.verbose);
    }

    #[test]
    fn test_sync_result_statistics() {
        let mut result = SyncResult::new();

        // Add some statistics
        result.synced_count = 5;
        result.cloned_count = 2;
        result.updated_count = 3;
        result.skipped_count = 1;

        assert_eq!(result.total_count(), 6); // cloned + updated + skipped = 6, not synced_count
        assert_eq!(result.cloned_count, 2);
        assert_eq!(result.updated_count, 3);
        assert_eq!(result.skipped_count, 1);
        assert!(result.is_success()); // No errors yet
    }

    #[test]
    fn test_sync_result_with_errors() {
        let mut result = SyncResult::new();

        result.add_error("Repository clone failed".to_string());
        result.add_error("Network timeout".to_string());

        assert!(!result.is_success());
        assert_eq!(result.errors.len(), 2);
        assert!(result
            .errors
            .contains(&"Repository clone failed".to_string()));
        assert!(result.errors.contains(&"Network timeout".to_string()));
    }

    #[test]
    fn test_sync_operation_enum() {
        let cloned = SyncOperation::Cloned;
        let updated = SyncOperation::Updated;
        let skipped = SyncOperation::Skipped;

        assert_eq!(cloned, SyncOperation::Cloned);
        assert_ne!(cloned, updated);
        assert_ne!(updated, skipped);

        // Test debug formatting
        assert!(format!("{:?}", cloned).contains("Cloned"));
        assert!(format!("{:?}", updated).contains("Updated"));
        assert!(format!("{:?}", skipped).contains("Skipped"));
    }

    #[test]
    fn test_sync_repositories_error_types() {
        // Test error creation and formatting
        let workspace_error =
            SyncRepositoriesError::WorkspaceNotInitialized("/tmp/workspace".to_string());
        assert!(workspace_error.to_string().contains("not initialized"));

        let clone_error = SyncRepositoriesError::RepositoryCloneFailed("Network error".to_string());
        assert!(clone_error.to_string().contains("Repository clone failed"));

        let remote_update_error = SyncRepositoriesError::RemoteUpdateFailed {
            repo: "example/repo".to_string(),
            error: "Merge conflict".to_string(),
        };
        assert!(remote_update_error
            .to_string()
            .contains("Remote update failed"));
        assert!(remote_update_error.to_string().contains("example/repo"));

        let branch_sync_error = SyncRepositoriesError::BranchSyncFailed {
            repo: "example/repo".to_string(),
            error: "Fast-forward failed".to_string(),
        };
        assert!(branch_sync_error.to_string().contains("Branch sync failed"));
        assert!(branch_sync_error.to_string().contains("example/repo"));

        let git_op_error = SyncRepositoriesError::GitOperationFailed("Git error".to_string());
        assert!(git_op_error.to_string().contains("Git operation failed"));

        let manifest_error =
            SyncRepositoriesError::ManifestUpdateFailed("YAML parse error".to_string());
        assert!(manifest_error
            .to_string()
            .contains("Manifest update failed"));
    }
}
