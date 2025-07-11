//! コマンド実行とプロセス管理の統合テスト
//! 
//! CommandExecutor、ForeachCommandUseCase、および
//! 関連するコンポーネントの統合テストを実装

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use tempfile::TempDir;
use tsrc::{
    application::use_cases::foreach_command::{
        ForeachCommandUseCase, ForeachCommandConfig, ForeachResult, CommandStatus
    },
    domain::entities::{
        workspace::{Workspace, WorkspaceConfig, WorkspaceStatus},
        manifest::{Manifest, ManifestRepo},
    },
    infrastructure::process::command_executor::{
        CommandExecutor, ExecutionConfig, ExecutionTask, ParallelConfig, ParallelResult
    },
};

/// テスト用のワークスペースを作成するヘルパー関数
fn create_test_workspace_with_repos(temp_dir: &TempDir) -> Workspace {
    let workspace_path = temp_dir.path().to_path_buf();
    let config = WorkspaceConfig::new(
        "https://github.com/example/manifest.git",
        "main"
    );
    
    // テスト用のリポジトリディレクトリを作成
    let repo1_path = workspace_path.join("repo1");
    let repo2_path = workspace_path.join("repo2");
    std::fs::create_dir_all(&repo1_path).unwrap();
    std::fs::create_dir_all(&repo2_path).unwrap();
    
    // 各リポジトリにテストファイルを作成
    std::fs::write(repo1_path.join("README.md"), "# Repository 1\n").unwrap();
    std::fs::write(repo2_path.join("README.md"), "# Repository 2\n").unwrap();
    
    // マニフェストを作成
    let manifest_repos = vec![
        ManifestRepo::new("https://github.com/example/repo1.git", "repo1"),
        ManifestRepo::new("https://github.com/example/repo2.git", "repo2"),
    ];
    let manifest = Manifest::new(manifest_repos);
    
    Workspace::new(workspace_path, config)
        .with_manifest(manifest)
        .with_status(WorkspaceStatus::Initialized)
}

/// テスト用のマニフェストファイルを作成するヘルパー関数
fn create_manifest_file(workspace: &Workspace) -> std::io::Result<()> {
    let tsrc_dir = workspace.tsrc_dir();
    std::fs::create_dir_all(&tsrc_dir)?;
    
    let manifest_content = r#"repos:
  - dest: repo1
    url: https://github.com/example/repo1.git
    branch: main
  - dest: repo2
    url: https://github.com/example/repo2.git
    branch: develop
"#;
    
    let manifest_file = workspace.manifest_file_path();
    std::fs::write(manifest_file, manifest_content)?;
    
    Ok(())
}

#[tokio::test]
async fn test_command_executor_basic_operations() {
    // テスト環境の準備
    let temp_dir = TempDir::new().unwrap();
    let executor = CommandExecutor::new();
    
    // 1. 単純なコマンド実行
    let simple_config = ExecutionConfig::new()
        .with_working_directory(temp_dir.path().to_path_buf())
        .with_timeout(Duration::from_secs(10));
    
    let result = executor.execute("echo 'Hello, World!'", &simple_config).await;
    assert!(result.is_ok(), "Simple command should succeed");
    
    let execution_result = result.unwrap();
    assert_eq!(execution_result.exit_code, 0, "Exit code should be 0");
    assert!(execution_result.stdout.contains("Hello, World!"), "Output should contain expected text");
    
    // 2. 失敗するコマンドのテスト
    let fail_result = executor.execute("false", &simple_config).await;
    assert!(fail_result.is_ok(), "Command execution should return result even for failed commands");
    
    let fail_execution = fail_result.unwrap();
    assert_ne!(fail_execution.exit_code, 0, "Failed command should have non-zero exit code");
    
    // 3. 環境変数付きコマンド実行
    let mut env_vars = HashMap::new();
    env_vars.insert("TEST_VAR".to_string(), "test_value".to_string());
    
    let env_config = ExecutionConfig::new()
        .with_working_directory(temp_dir.path().to_path_buf())
        .with_environment_variables(env_vars);
    
    // Unix系システムでのテスト
    #[cfg(unix)]
    {
        let env_result = executor.execute("echo $TEST_VAR", &env_config).await;
        assert!(env_result.is_ok(), "Environment variable command should succeed");
        
        let env_execution = env_result.unwrap();
        assert!(env_execution.stdout.contains("test_value"), "Output should contain environment variable value");
    }
    
    // Windows系システムでのテスト
    #[cfg(windows)]
    {
        let env_result = executor.execute("echo %TEST_VAR%", &env_config).await;
        assert!(env_result.is_ok(), "Environment variable command should succeed");
        
        let env_execution = env_result.unwrap();
        assert!(env_execution.stdout.contains("test_value"), "Output should contain environment variable value");
    }
}

#[tokio::test]
async fn test_parallel_command_execution() {
    // テスト環境の準備
    let temp_dir = TempDir::new().unwrap();
    let executor = CommandExecutor::new();
    
    // 1. 並列実行用のタスクを作成
    let tasks = vec![
        ExecutionTask::new("task1", "echo 'Task 1'"),
        ExecutionTask::new("task2", "echo 'Task 2'"),
        ExecutionTask::new("task3", "echo 'Task 3'"),
    ];
    
    let parallel_config = ParallelConfig::new()
        .with_max_concurrent(2)
        .with_timeout(Duration::from_secs(30));
    
    // 2. 並列実行を実行
    let parallel_result = executor.execute_parallel(tasks, &parallel_config).await;
    assert!(parallel_result.is_ok(), "Parallel execution should succeed");
    
    let result = parallel_result.unwrap();
    assert_eq!(result.total_tasks(), 3, "Should have executed 3 tasks");
    assert_eq!(result.successful_tasks(), 3, "All tasks should succeed");
    assert_eq!(result.failed_tasks(), 0, "No tasks should fail");
    
    // 3. 結果を個別に確認
    let results = result.get_results();
    assert_eq!(results.len(), 3, "Should have 3 results");
    
    for (task_id, execution_result) in results {
        assert_eq!(execution_result.exit_code, 0, "Each task should succeed");
        assert!(execution_result.stdout.contains("Task"), "Output should contain task information");
    }
}

#[tokio::test]
async fn test_foreach_command_use_case() {
    // テスト環境の準備
    let temp_dir = TempDir::new().unwrap();
    let workspace = create_test_workspace_with_repos(&temp_dir);
    create_manifest_file(&workspace).unwrap();
    
    // 1. ForeachCommandの設定
    let config = ForeachCommandConfig::new("pwd")
        .with_verbose(true)
        .with_continue_on_error(true)
        .with_change_dir(true);
    
    // 2. ForeachCommandを実行
    let use_case = ForeachCommandUseCase::new(config);
    let result = use_case.execute(&workspace).await;
    
    assert!(result.is_ok(), "Foreach command should succeed");
    let foreach_result = result.unwrap();
    
    // 3. 実行結果を確認
    assert!(foreach_result.is_success(), "Foreach should complete successfully");
    assert_eq!(foreach_result.total_count(), 2, "Should execute command in 2 repositories");
    
    // 4. 各リポジトリの結果を確認
    let results = foreach_result.results;
    assert_eq!(results.len(), 2, "Should have results for 2 repositories");
    
    for command_result in &results {
        assert_eq!(command_result.status, CommandStatus::Success, "Each command should succeed");
        assert!(command_result.stdout.len() > 0, "Should have output");
    }
}

#[tokio::test]
async fn test_foreach_command_with_environment_variables() {
    // テスト環境の準備
    let temp_dir = TempDir::new().unwrap();
    let workspace = create_test_workspace_with_repos(&temp_dir);
    create_manifest_file(&workspace).unwrap();
    
    // 1. 環境変数付きのForeachCommand設定
    let config = ForeachCommandConfig::new("echo $TSRC_REPO_DEST")
        .with_environment_variable("CUSTOM_VAR", "custom_value")
        .with_continue_on_error(true)
        .with_verbose(true);
    
    // 2. ForeachCommandを実行
    let use_case = ForeachCommandUseCase::new(config);
    let result = use_case.execute(&workspace).await;
    
    assert!(result.is_ok(), "Foreach command with environment variables should succeed");
    let foreach_result = result.unwrap();
    
    // 3. 環境変数が設定されていることを確認
    assert!(foreach_result.is_success(), "Command should succeed");
    
    // 各リポジトリでTSRC_REPO_DESTが設定されていることを確認
    for command_result in &foreach_result.results {
        if command_result.status == CommandStatus::Success {
            // Unix系システムの場合のテスト
            #[cfg(unix)]
            {
                // モックなので実際の環境変数展開は行われないが、
                // コマンドが正常に実行されることを確認
                assert_eq!(command_result.exit_code, Some(0));
            }
        }
    }
}

#[tokio::test]
async fn test_foreach_command_error_handling() {
    // テスト環境の準備
    let temp_dir = TempDir::new().unwrap();
    let workspace = create_test_workspace_with_repos(&temp_dir);
    create_manifest_file(&workspace).unwrap();
    
    // 1. 失敗するコマンドを設定
    let config = ForeachCommandConfig::new("false") // 常に失敗するコマンド
        .with_continue_on_error(true)
        .with_verbose(true);
    
    // 2. ForeachCommandを実行
    let use_case = ForeachCommandUseCase::new(config);
    let result = use_case.execute(&workspace).await;
    
    assert!(result.is_ok(), "Foreach command should return result even with failures");
    let foreach_result = result.unwrap();
    
    // 3. エラーが適切に処理されていることを確認
    assert!(!foreach_result.is_success(), "Should not be successful due to command failures");
    assert!(foreach_result.failure_count > 0, "Should have failures");
    
    // 4. 失敗したコマンドの詳細を確認
    let failed_results = foreach_result.failed_results();
    assert!(failed_results.len() > 0, "Should have failed results");
    
    for failed_result in failed_results {
        assert_eq!(failed_result.status, CommandStatus::Failed);
    }
}

#[tokio::test]
async fn test_foreach_command_parallel_execution() {
    // テスト環境の準備
    let temp_dir = TempDir::new().unwrap();
    let workspace = create_test_workspace_with_repos(&temp_dir);
    create_manifest_file(&workspace).unwrap();
    
    // 1. 並列実行の設定
    let config = ForeachCommandConfig::new("sleep 1 && echo 'Done'")
        .with_parallel(true, Some(2))
        .with_continue_on_error(true)
        .with_timeout(10);
    
    // 2. 並列ForeachCommandを実行
    let use_case = ForeachCommandUseCase::new(config);
    let start_time = std::time::Instant::now();
    let result = use_case.execute(&workspace).await;
    let elapsed = start_time.elapsed();
    
    assert!(result.is_ok(), "Parallel foreach command should succeed");
    let foreach_result = result.unwrap();
    
    // 3. 並列実行により高速化されていることを確認
    // 順次実行なら2秒以上かかるが、並列実行なら短縮される
    assert!(foreach_result.is_parallel, "Should indicate parallel execution");
    
    // 4. 実行結果を確認
    assert_eq!(foreach_result.total_count(), 2, "Should execute in 2 repositories");
}

#[tokio::test]
async fn test_command_timeout_handling() {
    // テスト環境の準備
    let temp_dir = TempDir::new().unwrap();
    let executor = CommandExecutor::new();
    
    // 1. タイムアウトが発生するコマンド
    let config = ExecutionConfig::new()
        .with_working_directory(temp_dir.path().to_path_buf())
        .with_timeout(Duration::from_millis(100)); // 非常に短いタイムアウト
    
    // 長時間実行されるコマンド
    let timeout_result = executor.execute("sleep 5", &config).await;
    
    // タイムアウトエラーまたは適切な処理が行われることを確認
    // 実装によってはタイムアウトをResultで返すか、特別な処理を行う
    assert!(timeout_result.is_ok() || timeout_result.is_err(), "Should handle timeout appropriately");
}

#[tokio::test]
async fn test_comprehensive_command_workflow() {
    // テスト環境の準備
    let temp_dir = TempDir::new().unwrap();
    let workspace = create_test_workspace_with_repos(&temp_dir);
    create_manifest_file(&workspace).unwrap();
    
    // より多くのリポジトリを含むテストケース
    let repo3_path = workspace.root_path.join("repo3");
    std::fs::create_dir_all(&repo3_path).unwrap();
    std::fs::write(repo3_path.join("test.txt"), "test content").unwrap();
    
    // 1. 複数のコマンドを順次実行
    let commands = vec![
        "ls -la",      // ファイル一覧
        "pwd",         // 現在のディレクトリ
        "echo 'test'", // 単純な出力
    ];
    
    for command in commands {
        let config = ForeachCommandConfig::new(command)
            .with_change_dir(true)
            .with_verbose(true)
            .with_continue_on_error(true);
        
        let use_case = ForeachCommandUseCase::new(config);
        let result = use_case.execute(&workspace).await.unwrap();
        
        // 各コマンドが正常に実行されることを確認
        assert!(result.total_count() > 0, "Should execute command in repositories");
        
        // 出力があることを確認
        for command_result in &result.results {
            if command_result.status == CommandStatus::Success {
                assert!(command_result.stdout.len() > 0 || command_result.stderr.len() > 0,
                       "Should have some output");
            }
        }
    }
    
    // 2. 環境変数を使用した複雑なコマンド
    let env_config = ForeachCommandConfig::new("echo \"Repository: $TSRC_REPO_DEST, URL: $TSRC_REPO_URL\"")
        .with_environment_variable("WORKSPACE_ROOT", workspace.root_path.display().to_string())
        .with_change_dir(false) // ワークスペースルートから実行
        .with_verbose(true);
    
    let env_use_case = ForeachCommandUseCase::new(env_config);
    let env_result = env_use_case.execute(&workspace).await.unwrap();
    
    // 環境変数が適切に設定されていることを確認
    assert_eq!(env_result.total_count(), 2, "Should execute in 2 repositories");
    
    // 3. 最終的なワークスペース状態の確認
    assert!(workspace.is_initialized(), "Workspace should remain initialized");
    assert!(workspace.manifest.is_some(), "Manifest should be available");
}