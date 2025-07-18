use std::path::PathBuf;
use std::collections::HashMap;
use thiserror::Error;
use serde::{Deserialize, Serialize};
use futures::future::join_all;
use tokio::sync::Semaphore;
use std::sync::Arc;
use crate::domain::entities::{
    workspace::Workspace, 
    manifest::ManifestRepo,
};

/// ForeachCommand関連のエラー
#[derive(Debug, Error)]
pub enum ForeachCommandError {
    #[error("Workspace not initialized: {0}")]
    WorkspaceNotInitialized(String),
    
    #[error("Command execution failed for repo '{repo}': {error}")]
    CommandFailed { repo: String, error: String },
    
    #[error("Repository not found: {0}")]
    RepositoryNotFound(String),
    
    #[error("Command is empty or invalid")]
    InvalidCommand,
    
    #[error("Failed to set environment variable '{key}': {error}")]
    EnvironmentVariableError { key: String, error: String },
    
    #[error("Parallel execution failed: {0}")]
    ParallelExecutionFailed(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Process execution error: {0}")]
    ProcessError(String),
}

/// Foreach実行の設定
#[derive(Debug, Clone)]
pub struct ForeachCommandConfig {
    /// 実行するコマンド
    pub command: String,
    
    /// 特定のグループのみを対象にするか（Noneの場合は全て）
    pub groups: Option<Vec<String>>,
    
    /// 並列実行するか
    pub parallel: bool,
    
    /// 最大並列数（Noneの場合はCPU数）
    pub max_parallel: Option<usize>,
    
    /// エラーが発生した場合でも継続するか
    pub continue_on_error: bool,
    
    /// 詳細ログを出力するか
    pub verbose: bool,
    
    /// 追加の環境変数
    pub environment_variables: HashMap<String, String>,
    
    /// コマンドのタイムアウト（秒）
    pub timeout_seconds: Option<u64>,
    
    /// 作業ディレクトリをリポジトリルートに変更するか
    pub change_dir: bool,
}

impl Default for ForeachCommandConfig {
    fn default() -> Self {
        Self {
            command: String::new(),
            groups: None,
            parallel: false,
            max_parallel: None,
            continue_on_error: false,
            verbose: false,
            environment_variables: HashMap::new(),
            timeout_seconds: None,
            change_dir: true,
        }
    }
}

impl ForeachCommandConfig {
    /// 新しいForeachCommandConfigインスタンスを作成
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            ..Default::default()
        }
    }
    
    /// 並列実行を有効化
    pub fn with_parallel(mut self, parallel: bool, max_parallel: Option<usize>) -> Self {
        self.parallel = parallel;
        self.max_parallel = max_parallel;
        self
    }
    
    /// グループフィルタを設定
    pub fn with_groups(mut self, groups: Vec<String>) -> Self {
        self.groups = Some(groups);
        self
    }
    
    /// エラー継続フラグを設定
    pub fn with_continue_on_error(mut self, continue_on_error: bool) -> Self {
        self.continue_on_error = continue_on_error;
        self
    }
    
    /// 詳細ログを有効化
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }
    
    /// 環境変数を追加
    pub fn with_environment_variable(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.environment_variables.insert(key.into(), value.into());
        self
    }
    
    /// タイムアウトを設定
    pub fn with_timeout(mut self, timeout_seconds: u64) -> Self {
        self.timeout_seconds = Some(timeout_seconds);
        self
    }
    
    /// 作業ディレクトリ変更フラグを設定
    pub fn with_change_dir(mut self, change_dir: bool) -> Self {
        self.change_dir = change_dir;
        self
    }
}

/// コマンド実行の状態
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommandStatus {
    /// 成功
    Success,
    /// 失敗
    Failed,
    /// スキップ（リポジトリが存在しない等）
    Skipped,
    /// タイムアウト
    Timeout,
    /// 実行中（並列処理時）
    Running,
}

/// 単一リポジトリでのコマンド実行結果
#[derive(Debug, Clone)]
pub struct CommandResult {
    /// リポジトリの相対パス
    pub dest: String,
    
    /// 実行ステータス
    pub status: CommandStatus,
    
    /// 終了コード
    pub exit_code: Option<i32>,
    
    /// 標準出力
    pub stdout: String,
    
    /// 標準エラー出力
    pub stderr: String,
    
    /// 実行時間（ミリ秒）
    pub execution_time_ms: u64,
    
    /// エラーメッセージ（失敗時）
    pub error_message: Option<String>,
}

impl CommandResult {
    /// 新しいCommandResultを作成
    pub fn new(dest: String) -> Self {
        Self {
            dest,
            status: CommandStatus::Skipped,
            exit_code: None,
            stdout: String::new(),
            stderr: String::new(),
            execution_time_ms: 0,
            error_message: None,
        }
    }
    
    /// 成功状態に設定
    pub fn with_success(mut self, exit_code: i32, stdout: String, stderr: String, execution_time: u64) -> Self {
        self.status = CommandStatus::Success;
        self.exit_code = Some(exit_code);
        self.stdout = stdout;
        self.stderr = stderr;
        self.execution_time_ms = execution_time;
        self
    }
    
    /// 失敗状態に設定
    pub fn with_failure(mut self, exit_code: Option<i32>, error: String, execution_time: u64) -> Self {
        self.status = CommandStatus::Failed;
        self.exit_code = exit_code;
        self.error_message = Some(error);
        self.execution_time_ms = execution_time;
        self
    }
    
    /// タイムアウト状態に設定
    pub fn with_timeout(mut self, execution_time: u64) -> Self {
        self.status = CommandStatus::Timeout;
        self.execution_time_ms = execution_time;
        self.error_message = Some("Command timed out".to_string());
        self
    }
    
    /// スキップ状態に設定
    pub fn with_skip(mut self, reason: String) -> Self {
        self.status = CommandStatus::Skipped;
        self.error_message = Some(reason);
        self
    }
    
    /// 成功したかチェック
    pub fn is_success(&self) -> bool {
        matches!(self.status, CommandStatus::Success)
    }
    
    /// 失敗したかチェック
    pub fn is_failure(&self) -> bool {
        matches!(self.status, CommandStatus::Failed | CommandStatus::Timeout)
    }
}

/// 全体の実行結果
#[derive(Debug, Clone)]
pub struct ForeachResult {
    /// 各リポジトリでの実行結果
    pub results: Vec<CommandResult>,
    
    /// 成功したリポジトリ数
    pub success_count: usize,
    
    /// 失敗したリポジトリ数
    pub failure_count: usize,
    
    /// スキップされたリポジトリ数
    pub skipped_count: usize,
    
    /// 総実行時間（ミリ秒）
    pub total_execution_time_ms: u64,
    
    /// 並列実行されたか
    pub was_parallel: bool,
}

impl ForeachResult {
    pub fn new(was_parallel: bool) -> Self {
        Self {
            results: Vec::new(),
            success_count: 0,
            failure_count: 0,
            skipped_count: 0,
            total_execution_time_ms: 0,
            was_parallel,
        }
    }
    
    /// コマンド結果を追加
    pub fn add_result(&mut self, result: CommandResult) {
        match result.status {
            CommandStatus::Success => self.success_count += 1,
            CommandStatus::Failed | CommandStatus::Timeout => self.failure_count += 1,
            CommandStatus::Skipped => self.skipped_count += 1,
            CommandStatus::Running => {} // 実行中は集計しない
        }
        
        if !self.was_parallel {
            self.total_execution_time_ms += result.execution_time_ms;
        }
        
        self.results.push(result);
    }
    
    /// 全体的に成功したかチェック
    pub fn is_success(&self) -> bool {
        self.failure_count == 0 && self.success_count > 0
    }
    
    /// 合計リポジトリ数
    pub fn total_count(&self) -> usize {
        self.results.len()
    }
    
    /// 失敗した結果のみを取得
    pub fn failed_results(&self) -> Vec<&CommandResult> {
        self.results.iter().filter(|r| r.is_failure()).collect()
    }
}

/// Foreach実行のユースケース
pub struct ForeachCommandUseCase {
    /// 設定
    config: ForeachCommandConfig,
}

impl ForeachCommandUseCase {
    /// 新しいForeachCommandUseCaseインスタンスを作成
    pub fn new(config: ForeachCommandConfig) -> Self {
        Self { config }
    }

    /// Foreach実行を実行
    pub async fn execute(&self, workspace: &Workspace) -> Result<ForeachResult, ForeachCommandError> {
        // 1. 入力検証
        self.validate_command()?;
        
        // 2. ワークスペースの初期化チェック
        self.check_workspace_initialized(workspace)?;
        
        // 3. 実行対象リポジトリの決定
        let target_repos = self.determine_target_repositories(workspace)?;
        
        // 4. 環境変数の準備
        let env_vars = self.prepare_environment_variables(workspace)?;
        
        // 5. コマンド実行（並列または順次）
        let result = if self.config.parallel {
            self.execute_parallel(&target_repos, workspace, &env_vars).await?
        } else {
            self.execute_sequential(&target_repos, workspace, &env_vars).await?
        };
        
        Ok(result)
    }
    
    /// コマンドの検証
    fn validate_command(&self) -> Result<(), ForeachCommandError> {
        if self.config.command.trim().is_empty() {
            return Err(ForeachCommandError::InvalidCommand);
        }
        Ok(())
    }
    
    /// ワークスペースが初期化済みかチェック
    fn check_workspace_initialized(&self, workspace: &Workspace) -> Result<(), ForeachCommandError> {
        if !workspace.is_initialized() {
            return Err(ForeachCommandError::WorkspaceNotInitialized(
                workspace.root_path.display().to_string()
            ));
        }
        Ok(())
    }
    
    /// 実行対象リポジトリの決定
    fn determine_target_repositories(&self, workspace: &Workspace) -> Result<Vec<ManifestRepo>, ForeachCommandError> {
        let manifest = workspace.manifest.as_ref().ok_or_else(|| {
            ForeachCommandError::ProcessError("Manifest not loaded".to_string())
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
    
    /// 環境変数の準備
    fn prepare_environment_variables(&self, workspace: &Workspace) -> Result<HashMap<String, String>, ForeachCommandError> {
        let mut env_vars = self.config.environment_variables.clone();
        
        // ワークスペース関連の環境変数を設定
        env_vars.insert("TSRC_WORKSPACE_ROOT".to_string(), 
                       workspace.root_path.display().to_string());
        
        if let Some(_manifest) = &workspace.manifest {
            env_vars.insert("TSRC_MANIFEST_URL".to_string(), 
                           workspace.config.manifest_url.clone());
            env_vars.insert("TSRC_MANIFEST_BRANCH".to_string(), 
                           workspace.config.manifest_branch.clone());
        }
        
        Ok(env_vars)
    }
    
    /// 順次実行
    async fn execute_sequential(
        &self,
        target_repos: &[ManifestRepo],
        workspace: &Workspace,
        env_vars: &HashMap<String, String>,
    ) -> Result<ForeachResult, ForeachCommandError> {
        let mut result = ForeachResult::new(false);
        
        for repo in target_repos {
            if self.config.verbose {
                println!("Executing command in repository: {}", repo.dest);
            }
            
            let command_result = self.execute_command_in_repo(repo, workspace, env_vars).await;
            
            match command_result {
                Ok(cmd_result) => {
                    result.add_result(cmd_result);
                }
                Err(e) => {
                    let failed_result = CommandResult::new(repo.dest.clone())
                        .with_failure(None, e.to_string(), 0);
                    result.add_result(failed_result);
                    
                    if !self.config.continue_on_error {
                        return Err(e);
                    }
                }
            }
        }
        
        Ok(result)
    }
    
    /// 並列実行
    async fn execute_parallel(
        &self,
        target_repos: &[ManifestRepo],
        workspace: &Workspace,
        env_vars: &HashMap<String, String>,
    ) -> Result<ForeachResult, ForeachCommandError> {
        let mut result = ForeachResult::new(true);
        
        if self.config.verbose {
            println!("Executing commands in {} repositories in parallel", target_repos.len());
        }
        
        let start_time = std::time::Instant::now();
        
        // 並列度を制限するためのセマフォ
        let max_parallel = self.config.max_parallel.unwrap_or_else(|| {
            std::cmp::min(target_repos.len(), num_cpus::get())
        });
        let semaphore = Arc::new(Semaphore::new(max_parallel));
        
        // 各リポジトリのタスクを作成
        let tasks: Vec<_> = target_repos.iter().map(|repo| {
            let repo = repo.clone();
            let workspace = workspace.clone();
            let env_vars = env_vars.clone();
            let semaphore = semaphore.clone();
            let config = self.config.clone();
            
            tokio::spawn(async move {
                let _permit = semaphore.acquire().await.map_err(|e| {
                    ForeachCommandError::ParallelExecutionFailed(format!("Failed to acquire semaphore: {}", e))
                })?;
                
                let use_case = ForeachCommandUseCase { config };
                use_case.execute_command_in_repo(&repo, &workspace, &env_vars).await
            })
        }).collect();
        
        // すべてのタスクを並列実行
        let results = join_all(tasks).await;
        
        // 結果をまとめる
        for (i, join_result) in results.into_iter().enumerate() {
            match join_result {
                Ok(task_result) => {
                    match task_result {
                        Ok(cmd_result) => {
                            result.add_result(cmd_result);
                        }
                        Err(e) => {
                            let repo_name = target_repos.get(i)
                                .map(|r| r.dest.clone())
                                .unwrap_or_else(|| "unknown".to_string());
                            let failed_result = CommandResult::new(repo_name)
                                .with_failure(None, e.to_string(), 0);
                            result.add_result(failed_result);
                            
                            if !self.config.continue_on_error {
                                return Err(e);
                            }
                        }
                    }
                }
                Err(join_err) => {
                    let repo_name = target_repos.get(i)
                        .map(|r| r.dest.clone())
                        .unwrap_or_else(|| "unknown".to_string());
                    let failed_result = CommandResult::new(repo_name)
                        .with_failure(None, format!("Task join error: {}", join_err), 0);
                    result.add_result(failed_result);
                    
                    if !self.config.continue_on_error {
                        return Err(ForeachCommandError::ParallelExecutionFailed(
                            format!("Task join error: {}", join_err)
                        ));
                    }
                }
            }
        }
        
        // 並列実行の場合は全体の実行時間を設定
        result.total_execution_time_ms = start_time.elapsed().as_millis() as u64;
        
        Ok(result)
    }
    
    /// 単一リポジトリでコマンド実行
    async fn execute_command_in_repo(
        &self,
        repo: &ManifestRepo,
        workspace: &Workspace,
        env_vars: &HashMap<String, String>,
    ) -> Result<CommandResult, ForeachCommandError> {
        let repo_path = workspace.repo_path(&repo.dest);
        
        // リポジトリが存在しない場合はスキップ
        if !repo_path.exists() {
            return Ok(CommandResult::new(repo.dest.clone())
                .with_skip("Repository directory does not exist".to_string()));
        }
        
        // 作業ディレクトリの決定
        let working_dir = if self.config.change_dir {
            repo_path.clone()
        } else {
            workspace.root_path.clone()
        };
        
        // リポジトリ固有の環境変数を追加
        let mut repo_env_vars = env_vars.clone();
        repo_env_vars.insert("TSRC_REPO_DEST".to_string(), repo.dest.clone());
        repo_env_vars.insert("TSRC_REPO_URL".to_string(), repo.url.clone());
        repo_env_vars.insert("TSRC_REPO_PATH".to_string(), repo_path.display().to_string());
        
        if let Some(branch) = &repo.branch {
            repo_env_vars.insert("TSRC_REPO_BRANCH".to_string(), branch.clone());
        }
        
        // コマンド実行
        self.perform_command_execution(&self.config.command, &working_dir, &repo_env_vars, &repo.dest).await
    }
    
    /// 実際のコマンド実行（疑似実装）
    async fn perform_command_execution(
        &self,
        command: &str,
        working_dir: &PathBuf,
        env_vars: &HashMap<String, String>,
        repo_dest: &str,
    ) -> Result<CommandResult, ForeachCommandError> {
        let start_time = std::time::Instant::now();
        
        if self.config.verbose {
            println!("Running '{}' in {}", command, working_dir.display());
            println!("Environment variables: {:?}", env_vars);
        }
        
        // TODO: 実際のプロセス実行はインフラストラクチャ層で実装
        // ここでは疑似的な実装
        
        // コマンド解析（簡易版）
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return Err(ForeachCommandError::InvalidCommand);
        }
        
        // タイムアウト処理のシミュレーション
        if let Some(timeout) = self.config.timeout_seconds {
            let elapsed = start_time.elapsed().as_secs();
            if elapsed >= timeout {
                return Ok(CommandResult::new(repo_dest.to_string())
                    .with_timeout(start_time.elapsed().as_millis() as u64));
            }
        }
        
        // 疑似的な成功結果を返す
        let execution_time = start_time.elapsed().as_millis() as u64;
        let result = CommandResult::new(repo_dest.to_string())
            .with_success(
                0, 
                format!("Mock output from '{}' in {}", command, repo_dest),
                String::new(),
                execution_time
            );
        
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use crate::domain::entities::workspace::WorkspaceConfig;
    use crate::domain::entities::manifest::{Manifest, ManifestRepo};
    
    #[test]
    fn test_foreach_config_creation() {
        let config = ForeachCommandConfig::new("git status")
            .with_parallel(true, Some(4))
            .with_groups(vec!["group1".to_string()])
            .with_continue_on_error(true)
            .with_verbose(true)
            .with_environment_variable("TEST_VAR", "test_value")
            .with_timeout(30);
        
        assert_eq!(config.command, "git status");
        assert!(config.parallel);
        assert_eq!(config.max_parallel, Some(4));
        assert!(config.continue_on_error);
        assert!(config.verbose);
        assert_eq!(config.environment_variables.get("TEST_VAR"), Some(&"test_value".to_string()));
        assert_eq!(config.timeout_seconds, Some(30));
    }
    
    #[test]
    fn test_command_result_creation() {
        let result = CommandResult::new("test/repo".to_string());
        assert_eq!(result.dest, "test/repo");
        assert_eq!(result.status, CommandStatus::Skipped);
        assert!(!result.is_success());
        assert!(!result.is_failure());
    }
    
    #[test]
    fn test_command_result_status() {
        let success_result = CommandResult::new("repo".to_string())
            .with_success(0, "output".to_string(), String::new(), 100);
        assert!(success_result.is_success());
        assert!(!success_result.is_failure());
        assert_eq!(success_result.exit_code, Some(0));
        
        let failed_result = CommandResult::new("repo".to_string())
            .with_failure(Some(1), "error".to_string(), 100);
        assert!(!failed_result.is_success());
        assert!(failed_result.is_failure());
        assert_eq!(failed_result.exit_code, Some(1));
        
        let timeout_result = CommandResult::new("repo".to_string())
            .with_timeout(1000);
        assert!(!timeout_result.is_success());
        assert!(timeout_result.is_failure());
        assert_eq!(timeout_result.status, CommandStatus::Timeout);
        
        let skipped_result = CommandResult::new("repo".to_string())
            .with_skip("No repo".to_string());
        assert!(!skipped_result.is_success());
        assert!(!skipped_result.is_failure());
        assert_eq!(skipped_result.status, CommandStatus::Skipped);
    }
    
    #[test]
    fn test_foreach_result_counting() {
        let mut result = ForeachResult::new(false);
        
        let success_result = CommandResult::new("success".to_string())
            .with_success(0, "output".to_string(), String::new(), 100);
        let failed_result = CommandResult::new("failed".to_string())
            .with_failure(Some(1), "error".to_string(), 200);
        let skipped_result = CommandResult::new("skipped".to_string())
            .with_skip("No repo".to_string());
        
        result.add_result(success_result);
        result.add_result(failed_result);
        result.add_result(skipped_result);
        
        assert_eq!(result.success_count, 1);
        assert_eq!(result.failure_count, 1);
        assert_eq!(result.skipped_count, 1);
        assert_eq!(result.total_count(), 3);
        assert!(!result.is_success()); // failure_count > 0 なので false
        assert_eq!(result.total_execution_time_ms, 300); // 100 + 200 + 0
        
        let failed_results = result.failed_results();
        assert_eq!(failed_results.len(), 1);
        assert_eq!(failed_results[0].dest, "failed");
    }
    
    #[test]
    fn test_command_validation() {
        let valid_config = ForeachCommandConfig::new("git status");
        let use_case = ForeachCommandUseCase::new(valid_config);
        assert!(use_case.validate_command().is_ok());
        
        let empty_config = ForeachCommandConfig::new("");
        let use_case = ForeachCommandUseCase::new(empty_config);
        assert!(use_case.validate_command().is_err());
        
        let whitespace_config = ForeachCommandConfig::new("   ");
        let use_case = ForeachCommandUseCase::new(whitespace_config);
        assert!(use_case.validate_command().is_err());
    }
    
    #[tokio::test]
    async fn test_workspace_initialization_check() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_config = WorkspaceConfig::new("https://example.com/manifest.git", "main");
        let workspace = Workspace::new(temp_dir.path().to_path_buf(), workspace_config);
        
        let config = ForeachCommandConfig::new("git status");
        let use_case = ForeachCommandUseCase::new(config);
        
        // 未初期化の場合はエラー
        let result = use_case.check_workspace_initialized(&workspace);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_target_repositories_determination() {
        let repos = vec![
            ManifestRepo::new("git@github.com:example/repo1.git", "repo1"),
            ManifestRepo::new("git@github.com:example/repo2.git", "repo2"),
        ];
        let manifest = Manifest::new(repos);
        let temp_dir = TempDir::new().unwrap();
        let workspace_config = WorkspaceConfig::new("https://example.com/manifest.git", "main");
        let workspace = Workspace::new(temp_dir.path().to_path_buf(), workspace_config)
            .with_manifest(manifest);
        
        let config = ForeachCommandConfig::new("git status");
        let use_case = ForeachCommandUseCase::new(config);
        
        let result = use_case.determine_target_repositories(&workspace);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 2);
    }
    
    #[test]
    fn test_environment_variables_preparation() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_config = WorkspaceConfig::new("https://example.com/manifest.git", "main");
        let manifest = Manifest::new(vec![]);
        let workspace = Workspace::new(temp_dir.path().to_path_buf(), workspace_config)
            .with_manifest(manifest);
        
        let config = ForeachCommandConfig::new("git status")
            .with_environment_variable("CUSTOM_VAR", "custom_value");
        let use_case = ForeachCommandUseCase::new(config);
        
        let env_vars = use_case.prepare_environment_variables(&workspace).unwrap();
        
        assert!(env_vars.contains_key("TSRC_WORKSPACE_ROOT"));
        assert!(env_vars.contains_key("TSRC_MANIFEST_URL"));
        assert!(env_vars.contains_key("TSRC_MANIFEST_BRANCH"));
        assert_eq!(env_vars.get("CUSTOM_VAR"), Some(&"custom_value".to_string()));
        assert_eq!(env_vars.get("TSRC_MANIFEST_URL"), Some(&"https://example.com/manifest.git".to_string()));
    }
}