use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command as TokioCommand};
use tokio::sync::Semaphore;
use tokio::time::timeout;

/// Command executor errors
#[derive(Debug, Error)]
pub enum CommandExecutorError {
    #[error("Command failed with exit code {exit_code}: {stderr}")]
    CommandFailed {
        exit_code: i32,
        stderr: String,
    },
    
    #[error("Command timed out after {timeout_seconds} seconds")]
    Timeout {
        timeout_seconds: u64,
    },
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Invalid command: {0}")]
    InvalidCommand(String),
    
    #[error("Process spawn failed: {0}")]
    SpawnFailed(String),
    
    #[error("Process termination failed: {0}")]
    TerminationFailed(String),
}

/// Configuration for command execution
#[derive(Debug, Clone)]
pub struct ExecutionConfig {
    /// Working directory for command execution
    pub working_directory: Option<PathBuf>,
    
    /// Environment variables to set for the process
    pub environment_variables: HashMap<String, String>,
    
    /// Timeout for command execution in seconds
    pub timeout_seconds: Option<u64>,
    
    /// Whether to capture stdout
    pub capture_stdout: bool,
    
    /// Whether to capture stderr
    pub capture_stderr: bool,
    
    /// Whether to inherit parent process environment
    pub inherit_environment: bool,
    
    /// Whether to run the command in a shell
    pub use_shell: bool,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            working_directory: None,
            environment_variables: HashMap::new(),
            timeout_seconds: None,
            capture_stdout: true,
            capture_stderr: true,
            inherit_environment: true,
            use_shell: false,
        }
    }
}

impl ExecutionConfig {
    /// Create a new execution config
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set working directory
    pub fn with_working_directory<P: AsRef<Path>>(mut self, dir: P) -> Self {
        self.working_directory = Some(dir.as_ref().to_path_buf());
        self
    }
    
    /// Add environment variable
    pub fn with_environment_variable(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.environment_variables.insert(key.into(), value.into());
        self
    }
    
    /// Add multiple environment variables
    pub fn with_environment_variables(mut self, vars: HashMap<String, String>) -> Self {
        self.environment_variables.extend(vars);
        self
    }
    
    /// Set timeout
    pub fn with_timeout(mut self, timeout_seconds: u64) -> Self {
        self.timeout_seconds = Some(timeout_seconds);
        self
    }
    
    /// Set output capture flags
    pub fn with_output_capture(mut self, capture_stdout: bool, capture_stderr: bool) -> Self {
        self.capture_stdout = capture_stdout;
        self.capture_stderr = capture_stderr;
        self
    }
    
    /// Set whether to inherit parent environment
    pub fn with_inherit_environment(mut self, inherit: bool) -> Self {
        self.inherit_environment = inherit;
        self
    }
    
    /// Set whether to use shell for execution
    pub fn with_shell(mut self, use_shell: bool) -> Self {
        self.use_shell = use_shell;
        self
    }
}

/// Result of command execution
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// Exit code of the process
    pub exit_code: i32,
    
    /// Standard output
    pub stdout: String,
    
    /// Standard error output
    pub stderr: String,
    
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
    
    /// Whether the command was successful (exit code 0)
    pub success: bool,
}

impl ExecutionResult {
    /// Create a new execution result
    pub fn new(exit_code: i32, stdout: String, stderr: String, execution_time_ms: u64) -> Self {
        Self {
            exit_code,
            stdout,
            stderr,
            execution_time_ms,
            success: exit_code == 0,
        }
    }
    
    /// Create a timeout result
    pub fn timeout(execution_time_ms: u64) -> Self {
        Self {
            exit_code: -1,
            stdout: String::new(),
            stderr: "Command timed out".to_string(),
            execution_time_ms,
            success: false,
        }
    }
}

/// Parallel execution configuration
#[derive(Debug, Clone)]
pub struct ParallelConfig {
    /// Maximum number of concurrent executions
    pub max_concurrency: usize,
    
    /// Whether to stop on first error
    pub fail_fast: bool,
}

impl Default for ParallelConfig {
    fn default() -> Self {
        Self {
            max_concurrency: num_cpus::get(),
            fail_fast: false,
        }
    }
}

impl ParallelConfig {
    /// Create new parallel config
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Set maximum concurrency
    pub fn with_max_concurrency(mut self, max_concurrency: usize) -> Self {
        self.max_concurrency = max_concurrency.max(1);
        self
    }
    
    /// Set fail fast behavior
    pub fn with_fail_fast(mut self, fail_fast: bool) -> Self {
        self.fail_fast = fail_fast;
        self
    }
}

/// Task for parallel execution
#[derive(Debug, Clone)]
pub struct ExecutionTask {
    /// Unique identifier for the task
    pub id: String,
    
    /// Command to execute
    pub command: String,
    
    /// Execution configuration
    pub config: ExecutionConfig,
}

impl ExecutionTask {
    /// Create a new execution task
    pub fn new(id: impl Into<String>, command: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            command: command.into(),
            config: ExecutionConfig::default(),
        }
    }
    
    /// Set execution config
    pub fn with_config(mut self, config: ExecutionConfig) -> Self {
        self.config = config;
        self
    }
}

/// Result of parallel execution
#[derive(Debug)]
pub struct ParallelResult {
    /// Results of individual tasks
    pub task_results: HashMap<String, Result<ExecutionResult, CommandExecutorError>>,
    
    /// Total execution time in milliseconds
    pub total_execution_time_ms: u64,
    
    /// Number of successful tasks
    pub success_count: usize,
    
    /// Number of failed tasks
    pub failure_count: usize,
}

impl ParallelResult {
    /// Create new parallel result
    pub fn new() -> Self {
        Self {
            task_results: HashMap::new(),
            total_execution_time_ms: 0,
            success_count: 0,
            failure_count: 0,
        }
    }
    
    /// Add task result
    pub fn add_result(&mut self, task_id: String, result: Result<ExecutionResult, CommandExecutorError>) {
        match &result {
            Ok(exec_result) if exec_result.success => self.success_count += 1,
            _ => self.failure_count += 1,
        }
        self.task_results.insert(task_id, result);
    }
    
    /// Check if all tasks succeeded
    pub fn is_success(&self) -> bool {
        self.failure_count == 0 && self.success_count > 0
    }
    
    /// Get failed task results
    pub fn failed_results(&self) -> Vec<(&String, &CommandExecutorError)> {
        self.task_results
            .iter()
            .filter_map(|(id, result)| {
                if let Err(error) = result {
                    Some((id, error))
                } else {
                    None
                }
            })
            .collect()
    }
}

/// Command executor for running external processes
pub struct CommandExecutor;

impl CommandExecutor {
    /// Execute a single command
    pub async fn execute(
        command: &str,
        config: &ExecutionConfig,
    ) -> Result<ExecutionResult, CommandExecutorError> {
        let start_time = Instant::now();
        
        // Parse command
        let (program, args) = Self::parse_command(command, config.use_shell)?;
        
        // Build tokio command
        let mut cmd = TokioCommand::new(&program);
        cmd.args(&args);
        
        // Set working directory
        if let Some(working_dir) = &config.working_directory {
            cmd.current_dir(working_dir);
        }
        
        // Set environment variables
        if !config.inherit_environment {
            cmd.env_clear();
        }
        for (key, value) in &config.environment_variables {
            cmd.env(key, value);
        }
        
        // Set stdio configuration
        cmd.stdout(if config.capture_stdout { Stdio::piped() } else { Stdio::inherit() });
        cmd.stderr(if config.capture_stderr { Stdio::piped() } else { Stdio::inherit() });
        cmd.stdin(Stdio::null());
        
        // Spawn process
        let child = cmd.spawn()
            .map_err(|e| CommandExecutorError::SpawnFailed(format!("Failed to spawn '{}': {}", command, e)))?;
        
        // Execute with optional timeout
        let result = if let Some(timeout_secs) = config.timeout_seconds {
            let timeout_duration = Duration::from_secs(timeout_secs);
            match timeout(timeout_duration, Self::wait_for_completion(child)).await {
                Ok(result) => result,
                Err(_) => {
                    let _execution_time = start_time.elapsed().as_millis() as u64;
                    return Err(CommandExecutorError::Timeout { timeout_seconds: timeout_secs });
                }
            }
        } else {
            Self::wait_for_completion(child).await
        }?;
        
        let execution_time = start_time.elapsed().as_millis() as u64;
        
        Ok(ExecutionResult::new(
            result.exit_code,
            result.stdout,
            result.stderr,
            execution_time,
        ))
    }
    
    /// Execute multiple commands in parallel
    pub async fn execute_parallel(
        tasks: Vec<ExecutionTask>,
        parallel_config: &ParallelConfig,
    ) -> ParallelResult {
        let start_time = Instant::now();
        let mut result = ParallelResult::new();
        
        if tasks.is_empty() {
            result.total_execution_time_ms = start_time.elapsed().as_millis() as u64;
            return result;
        }
        
        // Create semaphore for concurrency control
        let semaphore = Arc::new(Semaphore::new(parallel_config.max_concurrency));
        
        // Create futures for all tasks
        let mut task_futures = Vec::new();
        
        for task in tasks {
            let sem = semaphore.clone();
            let task_id = task.id.clone();
            let command = task.command.clone();
            let config = task.config.clone();
            
            let future = async move {
                let _permit = sem.acquire().await.unwrap();
                let result = Self::execute(&command, &config).await;
                (task_id, result)
            };
            
            task_futures.push(future);
        }
        
        // Execute all tasks
        let results = futures::future::join_all(task_futures).await;
        
        // Process results
        for (task_id, task_result) in results {
            if parallel_config.fail_fast && task_result.is_err() {
                result.add_result(task_id, task_result);
                break;
            }
            result.add_result(task_id, task_result);
        }
        
        result.total_execution_time_ms = start_time.elapsed().as_millis() as u64;
        result
    }
    
    /// Parse command into program and arguments
    fn parse_command(command: &str, use_shell: bool) -> Result<(String, Vec<String>), CommandExecutorError> {
        if command.trim().is_empty() {
            return Err(CommandExecutorError::InvalidCommand("Command is empty".to_string()));
        }
        
        if use_shell {
            // Use shell to execute command
            let shell = if cfg!(target_os = "windows") {
                "cmd"
            } else {
                "sh"
            };
            
            let shell_flag = if cfg!(target_os = "windows") {
                "/C"
            } else {
                "-c"
            };
            
            Ok((shell.to_string(), vec![shell_flag.to_string(), command.to_string()]))
        } else {
            // Parse command manually
            let parts: Vec<&str> = command.split_whitespace().collect();
            if parts.is_empty() {
                return Err(CommandExecutorError::InvalidCommand("Command has no parts".to_string()));
            }
            
            let program = parts[0].to_string();
            let args = parts[1..].iter().map(|s| s.to_string()).collect();
            
            Ok((program, args))
        }
    }
    
    /// Wait for child process to complete and capture output
    async fn wait_for_completion(mut child: Child) -> Result<ExecutionResult, CommandExecutorError> {
        let mut stdout_data = String::new();
        let mut stderr_data = String::new();
        
        // Capture stdout if available
        if let Some(stdout) = child.stdout.take() {
            let mut reader = BufReader::new(stdout);
            let mut line = String::new();
            while reader.read_line(&mut line).await? > 0 {
                stdout_data.push_str(&line);
                line.clear();
            }
        }
        
        // Capture stderr if available
        if let Some(stderr) = child.stderr.take() {
            let mut reader = BufReader::new(stderr);
            let mut line = String::new();
            while reader.read_line(&mut line).await? > 0 {
                stderr_data.push_str(&line);
                line.clear();
            }
        }
        
        // Wait for process to exit
        let exit_status = child.wait().await
            .map_err(|e| CommandExecutorError::TerminationFailed(format!("Failed to wait for process: {}", e)))?;
        
        let exit_code = exit_status.code().unwrap_or(-1);
        
        Ok(ExecutionResult {
            exit_code,
            stdout: stdout_data,
            stderr: stderr_data,
            execution_time_ms: 0, // Will be set by caller
            success: exit_status.success(),
        })
    }
    
    /// Check if a command exists in PATH
    pub fn command_exists(command: &str) -> bool {
        Command::new("which")
            .arg(command)
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
    
    /// Get the current working directory
    pub fn current_directory() -> Result<PathBuf, CommandExecutorError> {
        std::env::current_dir().map_err(CommandExecutorError::IoError)
    }
    
    /// Create a simple execution task
    pub fn create_task(id: impl Into<String>, command: impl Into<String>) -> ExecutionTask {
        ExecutionTask::new(id, command)
    }
    
    /// Create execution config with working directory and environment variables
    pub fn create_config(
        working_dir: Option<PathBuf>,
        env_vars: HashMap<String, String>,
    ) -> ExecutionConfig {
        let mut config = ExecutionConfig::new();
        
        if let Some(dir) = working_dir {
            config = config.with_working_directory(dir);
        }
        
        config.with_environment_variables(env_vars)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_simple_command_execution() {
        let config = ExecutionConfig::new();
        let result = CommandExecutor::execute("echo 'Hello, World!'", &config).await;
        
        assert!(result.is_ok());
        let exec_result = result.unwrap();
        assert!(exec_result.success);
        assert_eq!(exec_result.exit_code, 0);
        assert!(exec_result.stdout.contains("Hello, World!"));
    }
    
    #[tokio::test]
    async fn test_command_with_working_directory() {
        let temp_dir = TempDir::new().unwrap();
        let config = ExecutionConfig::new()
            .with_working_directory(temp_dir.path());
        
        let result = CommandExecutor::execute("pwd", &config).await;
        
        assert!(result.is_ok());
        let exec_result = result.unwrap();
        assert!(exec_result.success);
        assert!(exec_result.stdout.contains(&temp_dir.path().to_string_lossy().to_string()));
    }
    
    #[tokio::test]
    async fn test_command_with_environment_variables() {
        let config = ExecutionConfig::new()
            .with_environment_variable("TEST_VAR", "test_value");
        
        let result = CommandExecutor::execute("env", &config).await;
        
        assert!(result.is_ok());
        let exec_result = result.unwrap();
        assert!(exec_result.success);
        assert!(exec_result.stdout.contains("TEST_VAR=test_value"));
    }
    
    #[tokio::test]
    async fn test_command_timeout() {
        let config = ExecutionConfig::new()
            .with_timeout(1);
        
        let result = CommandExecutor::execute("sleep 3", &config).await;
        
        assert!(result.is_err());
        if let Err(CommandExecutorError::Timeout { timeout_seconds }) = result {
            assert_eq!(timeout_seconds, 1);
        } else {
            panic!("Expected timeout error");
        }
    }
    
    #[tokio::test]
    async fn test_failed_command() {
        let config = ExecutionConfig::new();
        let result = CommandExecutor::execute("false", &config).await;
        
        assert!(result.is_ok());
        let exec_result = result.unwrap();
        assert!(!exec_result.success);
        assert_eq!(exec_result.exit_code, 1);
    }
    
    #[tokio::test]
    async fn test_parallel_execution() {
        let tasks = vec![
            ExecutionTask::new("task1", "echo 'Task 1'"),
            ExecutionTask::new("task2", "echo 'Task 2'"),
            ExecutionTask::new("task3", "echo 'Task 3'"),
        ];
        
        let parallel_config = ParallelConfig::new().with_max_concurrency(2);
        let result = CommandExecutor::execute_parallel(tasks, &parallel_config).await;
        
        assert_eq!(result.task_results.len(), 3);
        assert_eq!(result.success_count, 3);
        assert_eq!(result.failure_count, 0);
        assert!(result.is_success());
        
        for (task_id, task_result) in &result.task_results {
            assert!(task_result.is_ok());
            let exec_result = task_result.as_ref().unwrap();
            assert!(exec_result.success);
            assert!(exec_result.stdout.contains(&format!("Task {}", task_id.chars().last().unwrap())));
        }
    }
    
    #[tokio::test]
    async fn test_parallel_execution_with_failures() {
        let tasks = vec![
            ExecutionTask::new("success", "echo 'Success'"),
            ExecutionTask::new("failure", "false"),
        ];
        
        let parallel_config = ParallelConfig::new();
        let result = CommandExecutor::execute_parallel(tasks, &parallel_config).await;
        
        assert_eq!(result.task_results.len(), 2);
        assert_eq!(result.success_count, 1);
        assert_eq!(result.failure_count, 1);
        assert!(!result.is_success());
        
        let success_result = result.task_results.get("success").unwrap();
        assert!(success_result.is_ok());
        
        let failure_result = result.task_results.get("failure").unwrap();
        assert!(failure_result.is_ok());
        let exec_result = failure_result.as_ref().unwrap();
        assert!(!exec_result.success);
    }
    
    #[test]
    fn test_command_parsing() {
        let (program, args) = CommandExecutor::parse_command("git status --porcelain", false).unwrap();
        assert_eq!(program, "git");
        assert_eq!(args, vec!["status", "--porcelain"]);
        
        let (program, args) = CommandExecutor::parse_command("echo 'Hello World'", true).unwrap();
        if cfg!(target_os = "windows") {
            assert_eq!(program, "cmd");
            assert_eq!(args, vec!["/C", "echo 'Hello World'"]);
        } else {
            assert_eq!(program, "sh");
            assert_eq!(args, vec!["-c", "echo 'Hello World'"]);
        }
    }
    
    #[test]
    fn test_execution_result_creation() {
        let result = ExecutionResult::new(0, "output".to_string(), "".to_string(), 100);
        assert!(result.success);
        assert_eq!(result.exit_code, 0);
        assert_eq!(result.stdout, "output");
        assert_eq!(result.execution_time_ms, 100);
        
        let timeout_result = ExecutionResult::timeout(1000);
        assert!(!timeout_result.success);
        assert_eq!(timeout_result.exit_code, -1);
        assert_eq!(timeout_result.stderr, "Command timed out");
        assert_eq!(timeout_result.execution_time_ms, 1000);
    }
    
    #[test]
    fn test_parallel_config() {
        let config = ParallelConfig::new()
            .with_max_concurrency(4)
            .with_fail_fast(true);
        
        assert_eq!(config.max_concurrency, 4);
        assert!(config.fail_fast);
    }
    
    #[test]
    fn test_execution_config() {
        let temp_dir = TempDir::new().unwrap();
        let config = ExecutionConfig::new()
            .with_working_directory(temp_dir.path())
            .with_environment_variable("KEY", "value")
            .with_timeout(30)
            .with_output_capture(true, false)
            .with_inherit_environment(false)
            .with_shell(true);
        
        assert_eq!(config.working_directory, Some(temp_dir.path().to_path_buf()));
        assert_eq!(config.environment_variables.get("KEY"), Some(&"value".to_string()));
        assert_eq!(config.timeout_seconds, Some(30));
        assert!(config.capture_stdout);
        assert!(!config.capture_stderr);
        assert!(!config.inherit_environment);
        assert!(config.use_shell);
    }
}