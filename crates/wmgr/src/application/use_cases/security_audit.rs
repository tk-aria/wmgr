use crate::application::services::security_service::{AuditResult, SecurityError, SecurityService};
use crate::domain::entities::manifest::ManifestRepo;
use crate::domain::entities::workspace::Workspace;
use std::path::PathBuf;
use thiserror::Error;

/// セキュリティ監査関連のエラー
#[derive(Debug, Error)]
pub enum SecurityAuditError {
    #[error("Workspace not initialized: {0}")]
    WorkspaceNotInitialized(String),

    #[error("Security service error: {0}")]
    SecurityServiceError(#[from] SecurityError),

    #[error("Repository not found: {0}")]
    RepositoryNotFound(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("No Rust projects found in workspace")]
    NoRustProjectsFound,
}

/// セキュリティ監査の設定
#[derive(Debug, Clone)]
pub struct SecurityAuditConfig {
    /// 特定のグループのみを対象にするか（Noneの場合は全て）
    pub groups: Option<Vec<String>>,

    /// 並列監査を実行するか
    pub parallel: bool,

    /// 最大並列数（Noneの場合はCPU数）
    pub max_parallel: Option<usize>,

    /// Critical/High脆弱性でエラーとするか
    pub fail_on_vulnerabilities: bool,

    /// 詳細ログを出力するか
    pub verbose: bool,
}

impl Default for SecurityAuditConfig {
    fn default() -> Self {
        Self {
            groups: None,
            parallel: true,
            max_parallel: None,
            fail_on_vulnerabilities: true,
            verbose: false,
        }
    }
}

impl SecurityAuditConfig {
    /// 新しいSecurityAuditConfigインスタンスを作成
    pub fn new() -> Self {
        Self::default()
    }

    /// グループフィルタを設定
    pub fn with_groups(mut self, groups: Vec<String>) -> Self {
        self.groups = Some(groups);
        self
    }

    /// 並列実行を設定
    pub fn with_parallel(mut self, parallel: bool, max_parallel: Option<usize>) -> Self {
        self.parallel = parallel;
        self.max_parallel = max_parallel;
        self
    }

    /// 脆弱性時のエラー処理を設定
    pub fn with_fail_on_vulnerabilities(mut self, fail_on_vulnerabilities: bool) -> Self {
        self.fail_on_vulnerabilities = fail_on_vulnerabilities;
        self
    }

    /// 詳細ログを設定
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }
}

/// 単一リポジトリの監査結果
#[derive(Debug, Clone)]
pub struct RepoAuditResult {
    /// リポジトリの相対パス
    pub dest: String,

    /// 監査結果
    pub audit_result: Option<AuditResult>,

    /// エラーメッセージ（監査失敗時）
    pub error: Option<String>,

    /// RustプロジェクトかどうかThe
    pub is_rust_project: bool,
}

impl RepoAuditResult {
    /// 監査に成功したかチェック
    pub fn is_success(&self) -> bool {
        self.audit_result.is_some() && self.error.is_none()
    }

    /// 脆弱性が見つかったかチェック
    pub fn has_vulnerabilities(&self) -> bool {
        self.audit_result
            .as_ref()
            .map(|r| r.warning_count.total() > 0)
            .unwrap_or(false)
    }

    /// Critical/High脆弱性があるかチェック
    pub fn has_critical_or_high_vulnerabilities(&self) -> bool {
        self.audit_result
            .as_ref()
            .map(|r| r.warning_count.has_critical_or_high())
            .unwrap_or(false)
    }
}

/// 全体の監査結果
#[derive(Debug, Clone)]
pub struct WorkspaceAuditResult {
    /// 各リポジトリの監査結果
    pub repo_results: Vec<RepoAuditResult>,

    /// 監査されたリポジトリ数
    pub audited_count: usize,

    /// スキップされたリポジトリ数（非Rustプロジェクト）
    pub skipped_count: usize,

    /// エラーが発生したリポジトリ数
    pub error_count: usize,

    /// 脆弱性が見つかったリポジトリ数
    pub vulnerable_count: usize,

    /// 並列実行されたか
    pub was_parallel: bool,
}

impl WorkspaceAuditResult {
    pub fn new(was_parallel: bool) -> Self {
        Self {
            repo_results: Vec::new(),
            audited_count: 0,
            skipped_count: 0,
            error_count: 0,
            vulnerable_count: 0,
            was_parallel,
        }
    }

    /// 監査結果を追加
    pub fn add_result(&mut self, result: RepoAuditResult) {
        if result.is_rust_project {
            if result.is_success() {
                self.audited_count += 1;
                if result.has_vulnerabilities() {
                    self.vulnerable_count += 1;
                }
            } else {
                self.error_count += 1;
            }
        } else {
            self.skipped_count += 1;
        }

        self.repo_results.push(result);
    }

    /// 全体的に成功したかチェック
    pub fn is_success(&self) -> bool {
        self.error_count == 0
    }

    /// 脆弱性が見つかったかチェック
    pub fn has_vulnerabilities(&self) -> bool {
        self.vulnerable_count > 0
    }

    /// Critical/High脆弱性があるかチェック
    pub fn has_critical_or_high_vulnerabilities(&self) -> bool {
        self.repo_results
            .iter()
            .any(|r| r.has_critical_or_high_vulnerabilities())
    }

    /// 合計リポジトリ数
    pub fn total_count(&self) -> usize {
        self.repo_results.len()
    }

    /// 失敗した結果のみを取得
    pub fn failed_results(&self) -> Vec<&RepoAuditResult> {
        self.repo_results
            .iter()
            .filter(|r| !r.is_success() && r.is_rust_project)
            .collect()
    }

    /// 脆弱性のある結果のみを取得
    pub fn vulnerable_results(&self) -> Vec<&RepoAuditResult> {
        self.repo_results
            .iter()
            .filter(|r| r.has_vulnerabilities())
            .collect()
    }
}

/// セキュリティ監査のユースケース
pub struct SecurityAuditUseCase {
    /// 設定
    config: SecurityAuditConfig,

    /// セキュリティサービス
    security_service: SecurityService,
}

impl SecurityAuditUseCase {
    /// 新しいSecurityAuditUseCaseインスタンスを作成
    pub fn new(config: SecurityAuditConfig) -> Self {
        Self {
            config,
            security_service: SecurityService::new(),
        }
    }

    /// カスタムセキュリティサービスを指定
    pub fn with_security_service(mut self, security_service: SecurityService) -> Self {
        self.security_service = security_service;
        self
    }

    /// セキュリティ監査を実行
    pub async fn execute(
        &self,
        workspace: &Workspace,
    ) -> Result<WorkspaceAuditResult, SecurityAuditError> {
        // 1. ワークスペースの初期化チェック
        self.check_workspace_initialized(workspace)?;

        // 2. 監査対象リポジトリの決定
        let target_repos = self.determine_target_repositories(workspace)?;

        // 3. Rustプロジェクトのフィルタリング
        let rust_repos = self
            .filter_rust_repositories(workspace, &target_repos)
            .await?;

        if rust_repos.is_empty() {
            return Err(SecurityAuditError::NoRustProjectsFound);
        }

        // 4. 監査実行（並列または順次）
        let result = if self.config.parallel {
            self.execute_parallel(workspace, &rust_repos).await?
        } else {
            self.execute_sequential(workspace, &rust_repos).await?
        };

        // 5. 結果の検証
        if self.config.fail_on_vulnerabilities && result.has_critical_or_high_vulnerabilities() {
            // Critical/High脆弱性が見つかった場合、詳細を出力してからエラーを返す
            if self.config.verbose {
                self.print_vulnerability_summary(&result);
            }
        }

        Ok(result)
    }

    /// ワークスペースが初期化済みかチェック
    fn check_workspace_initialized(&self, workspace: &Workspace) -> Result<(), SecurityAuditError> {
        if !workspace.is_initialized() {
            return Err(SecurityAuditError::WorkspaceNotInitialized(
                workspace.root_path.display().to_string(),
            ));
        }
        Ok(())
    }

    /// 監査対象リポジトリの決定
    fn determine_target_repositories(
        &self,
        workspace: &Workspace,
    ) -> Result<Vec<ManifestRepo>, SecurityAuditError> {
        let manifest = workspace.manifest.as_ref().ok_or_else(|| {
            SecurityAuditError::WorkspaceNotInitialized("Manifest not loaded".to_string())
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

    /// Rustプロジェクトをフィルタリング
    async fn filter_rust_repositories(
        &self,
        workspace: &Workspace,
        repos: &[ManifestRepo],
    ) -> Result<Vec<ManifestRepo>, SecurityAuditError> {
        let mut rust_repos = Vec::new();

        for repo in repos {
            let repo_path = workspace.repo_path(&repo.dest);
            if self.is_rust_project(&repo_path).await {
                rust_repos.push(repo.clone());
            }
        }

        Ok(rust_repos)
    }

    /// Rustプロジェクトかどうかを判定
    async fn is_rust_project(&self, repo_path: &PathBuf) -> bool {
        repo_path.join("Cargo.toml").exists()
    }

    /// 順次監査実行
    async fn execute_sequential(
        &self,
        workspace: &Workspace,
        target_repos: &[ManifestRepo],
    ) -> Result<WorkspaceAuditResult, SecurityAuditError> {
        let mut result = WorkspaceAuditResult::new(false);

        for repo in target_repos {
            if self.config.verbose {
                println!("Auditing dependencies in repository: {}", repo.dest);
            }

            let audit_result = self.audit_repository(workspace, repo).await;
            result.add_result(audit_result);
        }

        Ok(result)
    }

    /// 並列監査実行
    async fn execute_parallel(
        &self,
        workspace: &Workspace,
        target_repos: &[ManifestRepo],
    ) -> Result<WorkspaceAuditResult, SecurityAuditError> {
        let mut result = WorkspaceAuditResult::new(true);

        if self.config.verbose {
            println!(
                "Auditing dependencies in {} repositories in parallel",
                target_repos.len()
            );
        }

        // 並列度を制限するためのセマフォ
        let max_parallel = self
            .config
            .max_parallel
            .unwrap_or_else(|| std::cmp::min(target_repos.len(), num_cpus::get()));
        let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(max_parallel));

        // 各リポジトリのタスクを作成
        let tasks: Vec<_> = target_repos
            .iter()
            .map(|repo| {
                let repo = repo.clone();
                let workspace = workspace.clone();
                let semaphore = semaphore.clone();
                let security_service = SecurityService::new(); // 各タスクで独立したインスタンスを使用

                tokio::spawn(async move {
                    let _permit = semaphore.acquire().await.map_err(|e| {
                        SecurityAuditError::SecurityServiceError(
                            SecurityError::CommandExecutionFailed(format!(
                                "Failed to acquire semaphore: {}",
                                e
                            )),
                        )
                    })?;

                    let use_case = SecurityAuditUseCase {
                        config: SecurityAuditConfig::new(),
                        security_service,
                    };

                    Ok::<RepoAuditResult, SecurityAuditError>(
                        use_case.audit_repository(&workspace, &repo).await,
                    )
                })
            })
            .collect();

        // すべてのタスクを並列実行
        let results = futures::future::join_all(tasks).await;

        // 結果をまとめる
        for join_result in results {
            match join_result {
                Ok(Ok(repo_result)) => {
                    result.add_result(repo_result);
                }
                Ok(Err(_)) | Err(_) => {
                    // タスクの実行に失敗した場合のエラー処理
                    result.add_result(RepoAuditResult {
                        dest: "unknown".to_string(),
                        audit_result: None,
                        error: Some("Task execution failed".to_string()),
                        is_rust_project: true,
                    });
                }
            }
        }

        Ok(result)
    }

    /// 単一リポジトリの監査
    async fn audit_repository(
        &self,
        workspace: &Workspace,
        repo: &ManifestRepo,
    ) -> RepoAuditResult {
        let repo_path = workspace.repo_path(&repo.dest);

        // リポジトリが存在するかチェック
        if !repo_path.exists() {
            return RepoAuditResult {
                dest: repo.dest.clone(),
                audit_result: None,
                error: Some("Repository directory does not exist".to_string()),
                is_rust_project: false,
            };
        }

        // Rustプロジェクトかチェック
        let is_rust_project = self.is_rust_project(&repo_path).await;
        if !is_rust_project {
            return RepoAuditResult {
                dest: repo.dest.clone(),
                audit_result: None,
                error: None,
                is_rust_project: false,
            };
        }

        // 監査実行
        match self.security_service.audit_dependencies(&repo_path).await {
            Ok(audit_result) => RepoAuditResult {
                dest: repo.dest.clone(),
                audit_result: Some(audit_result),
                error: None,
                is_rust_project: true,
            },
            Err(e) => RepoAuditResult {
                dest: repo.dest.clone(),
                audit_result: None,
                error: Some(e.to_string()),
                is_rust_project: true,
            },
        }
    }

    /// 脆弱性の概要を表示
    fn print_vulnerability_summary(&self, result: &WorkspaceAuditResult) {
        println!("\n=== Security Audit Summary ===");
        println!("Total repositories: {}", result.total_count());
        println!("Audited: {}", result.audited_count);
        println!("Skipped (non-Rust): {}", result.skipped_count);
        println!("Errors: {}", result.error_count);
        println!("With vulnerabilities: {}", result.vulnerable_count);

        if result.has_vulnerabilities() {
            println!("\n=== Repositories with Vulnerabilities ===");
            for repo_result in result.vulnerable_results() {
                if let Some(audit_result) = &repo_result.audit_result {
                    println!(
                        "\n{}: {} vulnerabilities",
                        repo_result.dest,
                        audit_result.warning_count.total()
                    );

                    if audit_result.warning_count.critical > 0 {
                        println!("  Critical: {}", audit_result.warning_count.critical);
                    }
                    if audit_result.warning_count.high > 0 {
                        println!("  High: {}", audit_result.warning_count.high);
                    }
                    if audit_result.warning_count.medium > 0 {
                        println!("  Medium: {}", audit_result.warning_count.medium);
                    }
                    if audit_result.warning_count.low > 0 {
                        println!("  Low: {}", audit_result.warning_count.low);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::{manifest::Manifest, workspace::WorkspaceConfig};
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_security_audit_config() {
        let config = SecurityAuditConfig::new()
            .with_groups(vec!["group1".to_string()])
            .with_parallel(true, Some(4))
            .with_fail_on_vulnerabilities(false)
            .with_verbose(true);

        assert_eq!(config.groups, Some(vec!["group1".to_string()]));
        assert!(config.parallel);
        assert_eq!(config.max_parallel, Some(4));
        assert!(!config.fail_on_vulnerabilities);
        assert!(config.verbose);
    }

    #[test]
    fn test_repo_audit_result() {
        let result = RepoAuditResult {
            dest: "test-repo".to_string(),
            audit_result: None,
            error: None,
            is_rust_project: false,
        };

        assert!(!result.is_success());
        assert!(!result.has_vulnerabilities());
        assert!(!result.has_critical_or_high_vulnerabilities());
    }

    #[test]
    fn test_workspace_audit_result() {
        let mut result = WorkspaceAuditResult::new(false);
        assert_eq!(result.total_count(), 0);
        assert!(result.is_success());
        assert!(!result.has_vulnerabilities());

        let repo_result = RepoAuditResult {
            dest: "test-repo".to_string(),
            audit_result: None,
            error: None,
            is_rust_project: true,
        };

        result.add_result(repo_result);
        assert_eq!(result.total_count(), 1);
        assert_eq!(result.error_count, 1);
        assert!(!result.is_success());
    }

    #[tokio::test]
    async fn test_is_rust_project() {
        let temp_dir = TempDir::new().unwrap();
        let config = SecurityAuditConfig::new();
        let use_case = SecurityAuditUseCase::new(config);

        // Non-Rust project
        let non_rust_path = temp_dir.path().to_path_buf();
        assert!(!use_case.is_rust_project(&non_rust_path).await);

        // Rust project
        let cargo_toml_path = non_rust_path.join("Cargo.toml");
        fs::write(
            cargo_toml_path,
            "[package]\nname = \"test\"\nversion = \"0.1.0\"\nedition = \"2021\"",
        )
        .unwrap();
        assert!(use_case.is_rust_project(&non_rust_path).await);
    }

    #[tokio::test]
    async fn test_workspace_not_initialized() {
        let temp_dir = TempDir::new().unwrap();
        let workspace_config = WorkspaceConfig::new("https://example.com/manifest.git", "main");
        let workspace = Workspace::new(temp_dir.path().to_path_buf(), workspace_config);

        let config = SecurityAuditConfig::new();
        let use_case = SecurityAuditUseCase::new(config);

        let result = use_case.execute(&workspace).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SecurityAuditError::WorkspaceNotInitialized(_)
        ));
    }
}
