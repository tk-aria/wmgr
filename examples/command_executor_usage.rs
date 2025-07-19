use std::collections::HashMap;
use tsrc::infrastructure::process::{
    CommandExecutor, ExecutionConfig, ExecutionTask, ParallelConfig,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Command Executor Usage Examples ===\n");

    // Example 1: Simple command execution
    println!("1. Simple command execution:");
    let config = ExecutionConfig::new();
    let result = CommandExecutor::execute("echo 'Hello from tsrc!'", &config).await?;

    println!("Exit code: {}", result.exit_code);
    println!("Output: {}", result.stdout.trim());
    println!("Execution time: {}ms\n", result.execution_time_ms);

    // Example 2: Command with working directory
    println!("2. Command with working directory:");
    let temp_dir = tempfile::tempdir()?;
    let config = ExecutionConfig::new().with_working_directory(temp_dir.path());

    let result = CommandExecutor::execute("pwd", &config).await?;
    println!("Working directory: {}", result.stdout.trim());
    println!();

    // Example 3: Command with environment variables
    println!("3. Command with environment variables:");
    let mut env_vars = HashMap::new();
    env_vars.insert("TSRC_EXAMPLE".to_string(), "Hello World".to_string());
    env_vars.insert("TSRC_VERSION".to_string(), "1.0.0".to_string());

    let config = ExecutionConfig::new().with_environment_variables(env_vars);

    let result = CommandExecutor::execute("env | grep TSRC", &config).await?;
    println!("Environment variables:");
    println!("{}", result.stdout.trim());
    println!();

    // Example 4: Command with timeout
    println!("4. Command with timeout (will timeout):");
    let config = ExecutionConfig::new().with_timeout(1); // 1 second timeout

    match CommandExecutor::execute("sleep 2", &config).await {
        Ok(_) => println!("Command completed successfully"),
        Err(e) => println!("Command failed: {}", e),
    }
    println!();

    // Example 5: Parallel execution
    println!("5. Parallel execution:");
    let tasks = vec![
        ExecutionTask::new("git_version", "git --version"),
        ExecutionTask::new("rust_version", "rustc --version"),
        ExecutionTask::new("system_info", "uname -a"),
    ];

    let parallel_config = ParallelConfig::new().with_max_concurrency(2);
    let parallel_result = CommandExecutor::execute_parallel(tasks, &parallel_config).await;

    println!("Parallel execution completed:");
    println!("Success count: {}", parallel_result.success_count);
    println!("Failure count: {}", parallel_result.failure_count);
    println!("Total time: {}ms", parallel_result.total_execution_time_ms);

    for (task_id, result) in &parallel_result.task_results {
        match result {
            Ok(exec_result) if exec_result.success => {
                println!("  {}: {}", task_id, exec_result.stdout.trim());
            }
            Ok(exec_result) => {
                println!(
                    "  {} failed with exit code: {}",
                    task_id, exec_result.exit_code
                );
            }
            Err(e) => {
                println!("  {} error: {}", task_id, e);
            }
        }
    }
    println!();

    // Example 6: Repository-like operations
    println!("6. Repository-like operations simulation:");
    let repo_dirs = vec!["repo1", "repo2", "repo3"];
    let base_dir = temp_dir.path();

    // Create mock repository directories
    for repo in &repo_dirs {
        let repo_path = base_dir.join(repo);
        std::fs::create_dir_all(&repo_path)?;

        // Initialize as git repo
        let config = ExecutionConfig::new().with_working_directory(&repo_path);

        let _ = CommandExecutor::execute("git init", &config).await;
        let _ = CommandExecutor::execute("git config user.name 'Test User'", &config).await;
        let _ = CommandExecutor::execute("git config user.email 'test@example.com'", &config).await;

        // Create a test file and commit
        std::fs::write(
            repo_path.join("README.md"),
            format!("# {}\n\nTest repository", repo),
        )?;
        let _ = CommandExecutor::execute("git add .", &config).await;
        let _ = CommandExecutor::execute("git commit -m 'Initial commit'", &config).await;
    }

    // Run git status in all repositories in parallel
    let mut tasks = Vec::new();
    for repo in &repo_dirs {
        let repo_path = base_dir.join(repo);
        let config = ExecutionConfig::new()
            .with_working_directory(&repo_path)
            .with_environment_variable("TSRC_REPO_NAME", *repo);

        let task =
            ExecutionTask::new(repo.to_string(), "git status --porcelain").with_config(config);
        tasks.push(task);
    }

    let parallel_config = ParallelConfig::new().with_max_concurrency(3);
    let repo_result = CommandExecutor::execute_parallel(tasks, &parallel_config).await;

    println!("Git status check results:");
    for (repo_name, result) in &repo_result.task_results {
        match result {
            Ok(exec_result) if exec_result.success => {
                if exec_result.stdout.trim().is_empty() {
                    println!("  {}: Clean working directory", repo_name);
                } else {
                    println!("  {}: Has changes\n{}", repo_name, exec_result.stdout);
                }
            }
            Ok(exec_result) => {
                println!(
                    "  {}: Git status failed (exit code: {})",
                    repo_name, exec_result.exit_code
                );
            }
            Err(e) => {
                println!("  {}: Error - {}", repo_name, e);
            }
        }
    }

    println!("\n=== Command Executor Examples Completed ===");
    Ok(())
}
