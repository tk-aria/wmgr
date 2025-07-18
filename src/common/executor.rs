use crate::common::error::TsrcError;
use crate::common::result::TsrcResult;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, oneshot, Semaphore};
use tokio::time::timeout;
use tracing::{debug, error, info, warn};

/// タスクの実行状態を表す列挙型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskStatus {
    /// 待機中
    Pending,
    /// 実行中
    Running,
    /// 成功
    Success,
    /// 失敗
    Failed,
    /// キャンセル済み
    Cancelled,
    /// タイムアウト
    Timeout,
}

/// タスクの実行結果
#[derive(Debug)]
pub struct TaskResult<T> {
    /// タスクID
    pub task_id: String,
    /// 実行結果
    pub result: Result<T, TsrcError>,
    /// 実行開始時刻
    pub started_at: Instant,
    /// 実行終了時刻
    pub completed_at: Instant,
    /// 実行にかかった時間
    pub duration: Duration,
    /// タスクの状態
    pub status: TaskStatus,
}

impl<T> TaskResult<T> {
    /// 成功したかどうかを判定
    pub fn is_success(&self) -> bool {
        self.status == TaskStatus::Success
    }

    /// 失敗したかどうかを判定
    pub fn is_failed(&self) -> bool {
        matches!(self.status, TaskStatus::Failed | TaskStatus::Timeout)
    }
}

/// タスクの進捗情報
#[derive(Debug, Clone)]
pub struct TaskProgress {
    /// タスクID
    pub task_id: String,
    /// 現在の進捗（0.0-1.0）
    pub progress: f64,
    /// 進捗メッセージ
    pub message: Option<String>,
    /// 推定残り時間
    pub estimated_remaining: Option<Duration>,
}

/// 実行可能なタスクを表すトレイト
#[async_trait::async_trait]
pub trait ExecutableTask: Send + Sync {
    type Output: Send + std::fmt::Debug + 'static;

    /// タスクIDを取得
    fn id(&self) -> &str;

    /// タスクを実行
    async fn execute(&self, progress_sender: Option<mpsc::UnboundedSender<TaskProgress>>) -> TsrcResult<Self::Output>;

    /// タスクの推定実行時間を取得（オプション）
    fn estimated_duration(&self) -> Option<Duration> {
        None
    }

    /// タスクの依存関係を取得（オプション）
    fn dependencies(&self) -> Vec<String> {
        Vec::new()
    }

    /// タスクの優先度を取得（オプション、低い値ほど高優先度）
    fn priority(&self) -> u32 {
        100
    }
}

/// タスク実行設定
#[derive(Debug, Clone)]
pub struct ExecutorConfig {
    /// 最大同時実行数
    pub max_concurrent: usize,
    /// タスクのタイムアウト時間
    pub task_timeout: Option<Duration>,
    /// プログレス報告間隔
    pub progress_interval: Duration,
    /// エラー時の自動リトライ回数
    pub retry_count: u32,
    /// リトライ間隔
    pub retry_interval: Duration,
    /// 依存関係解決のタイムアウト
    pub dependency_timeout: Duration,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            max_concurrent: num_cpus::get(),
            task_timeout: Some(Duration::from_secs(300)), // 5分
            progress_interval: Duration::from_millis(500),
            retry_count: 0,
            retry_interval: Duration::from_secs(1),
            dependency_timeout: Duration::from_secs(30),
        }
    }
}

/// タスク実行エンジン
pub struct TaskExecutor {
    config: ExecutorConfig,
    semaphore: Arc<Semaphore>,
    running_tasks: Arc<Mutex<HashMap<String, oneshot::Sender<()>>>>,
    progress_sender: Option<mpsc::UnboundedSender<TaskProgress>>,
}

impl TaskExecutor {
    /// 新しいタスク実行エンジンを作成
    pub fn new(config: ExecutorConfig) -> Self {
        let semaphore = Arc::new(Semaphore::new(config.max_concurrent));
        let running_tasks = Arc::new(Mutex::new(HashMap::new()));

        Self {
            config,
            semaphore,
            running_tasks,
            progress_sender: None,
        }
    }

    /// プログレス監視を有効にする
    pub fn with_progress_monitoring(mut self) -> (Self, mpsc::UnboundedReceiver<TaskProgress>) {
        let (sender, receiver) = mpsc::unbounded_channel();
        self.progress_sender = Some(sender);
        (self, receiver)
    }

    /// 単一のタスクを実行
    pub async fn execute_task<T: ExecutableTask>(&self, task: T) -> TsrcResult<TaskResult<T::Output>> {
        let task_id = task.id().to_string();
        let started_at = Instant::now();

        info!("Starting task: {}", task_id);

        // セマフォを取得（同時実行数制御）
        let _permit = self.semaphore.acquire().await
            .map_err(|e| TsrcError::internal_error(format!("Failed to acquire semaphore: {}", e)))?;

        // キャンセル用チャンネル
        let (cancel_tx, cancel_rx) = oneshot::channel();
        
        // 実行中タスクに登録
        {
            let mut running = self.running_tasks.lock().unwrap();
            running.insert(task_id.clone(), cancel_tx);
        }

        // タスク実行
        let execute_result = self.execute_with_timeout_and_retry(task, cancel_rx).await;

        // 実行中タスクから除去
        {
            let mut running = self.running_tasks.lock().unwrap();
            running.remove(&task_id);
        }

        let completed_at = Instant::now();
        let duration = completed_at - started_at;

        let status = match &execute_result {
            Ok(_) => TaskStatus::Success,
            Err(TsrcError::Timeout { .. }) => TaskStatus::Timeout,
            Err(TsrcError::Cancelled) => TaskStatus::Cancelled,
            Err(_) => TaskStatus::Failed,
        };

        info!("Task {} completed with status: {:?} in {:?}", task_id, status, duration);

        Ok(TaskResult {
            task_id,
            result: execute_result,
            started_at,
            completed_at,
            duration,
            status,
        })
    }

    /// 複数のタスクを並列実行
    pub async fn execute_tasks<T: ExecutableTask + 'static>(
        &self,
        tasks: Vec<T>,
    ) -> Vec<TaskResult<T::Output>> {
        let mut handles = Vec::new();

        for task in tasks {
            let executor = self.clone_for_task();
            let handle = tokio::spawn(async move {
                executor.execute_task(task).await
            });
            handles.push(handle);
        }

        let mut results = Vec::new();
        for handle in handles {
            match handle.await {
                Ok(Ok(result)) => results.push(result),
                Ok(Err(e)) => {
                    error!("Task execution failed: {}", e);
                    // エラーの場合でもダミーの結果を作成
                    results.push(TaskResult {
                        task_id: "unknown".to_string(),
                        result: Err(e),
                        started_at: Instant::now(),
                        completed_at: Instant::now(),
                        duration: Duration::from_secs(0),
                        status: TaskStatus::Failed,
                    });
                }
                Err(e) => {
                    error!("Task join failed: {}", e);
                }
            }
        }

        results
    }

    /// 依存関係を考慮したタスク実行
    pub async fn execute_with_dependencies<T: ExecutableTask + Clone + 'static>(
        &self,
        tasks: Vec<T>,
    ) -> TsrcResult<Vec<TaskResult<T::Output>>> {
        // 依存関係グラフを構築
        let dependency_graph = self.build_dependency_graph(&tasks)?;
        
        // トポロジカルソートで実行順序を決定
        let execution_order = self.topological_sort(&dependency_graph)?;
        
        // 実行順序に従ってタスクを実行
        let mut results = Vec::new();
        let mut completed_tasks = std::collections::HashSet::new();
        
        for task_id in execution_order {
            if let Some(task) = tasks.iter().find(|t| t.id() == task_id) {
                // 依存関係が満たされるまで待機
                self.wait_for_dependencies(task, &completed_tasks).await?;
                
                // タスクを実行（クローンして所有権を移動）
                let result = self.execute_task(task.clone()).await?;
                
                if result.is_success() {
                    completed_tasks.insert(task_id.clone());
                }
                
                results.push(result);
            }
        }
        
        Ok(results)
    }

    /// 全ての実行中タスクをキャンセル
    pub async fn cancel_all(&self) {
        let mut running = self.running_tasks.lock().unwrap();
        let tasks_to_cancel: Vec<_> = running.drain().collect();
        drop(running); // Mutexを早期解放
        
        for (task_id, cancel_tx) in tasks_to_cancel {
            debug!("Cancelling task: {}", task_id);
            let _ = cancel_tx.send(());
        }
    }

    /// 指定されたタスクをキャンセル
    pub async fn cancel_task(&self, task_id: &str) -> bool {
        let mut running = self.running_tasks.lock().unwrap();
        if let Some(cancel_tx) = running.remove(task_id) {
            debug!("Cancelling task: {}", task_id);
            cancel_tx.send(()).is_ok()
        } else {
            false
        }
    }

    // プライベートメソッド

    async fn execute_with_timeout_and_retry<T: ExecutableTask>(
        &self,
        task: T,
        mut cancel_rx: oneshot::Receiver<()>,
    ) -> TsrcResult<T::Output> {
        let mut attempt = 0;
        let max_attempts = self.config.retry_count + 1;

        while attempt < max_attempts {
            if attempt > 0 {
                info!("Retrying task {} (attempt {}/{})", task.id(), attempt + 1, max_attempts);
                tokio::time::sleep(self.config.retry_interval).await;
            }

            // タスク実行
            let task_future = task.execute(self.progress_sender.clone());

            // タイムアウトとキャンセル処理
            let result = if let Some(task_timeout) = self.config.task_timeout {
                tokio::select! {
                    result = timeout(task_timeout, task_future) => {
                        match result {
                            Ok(task_result) => task_result,
                            Err(_) => Err(TsrcError::timeout(task_timeout.as_secs())),
                        }
                    }
                    _ = &mut cancel_rx => {
                        warn!("Task {} was cancelled", task.id());
                        Err(TsrcError::Cancelled)
                    }
                }
            } else {
                tokio::select! {
                    result = task_future => result,
                    _ = &mut cancel_rx => {
                        warn!("Task {} was cancelled", task.id());
                        Err(TsrcError::Cancelled)
                    }
                }
            };

            match result {
                Ok(output) => return Ok(output),
                Err(TsrcError::Cancelled) => return Err(TsrcError::Cancelled),
                Err(TsrcError::Timeout { .. }) => {
                    if let Err(e) = result {
                        return Err(e);
                    }
                }
                Err(e) => {
                    attempt += 1;
                    if attempt >= max_attempts {
                        return Err(e);
                    }
                    warn!("Task {} failed (attempt {}): {}", task.id(), attempt, e);
                }
            }
        }

        unreachable!()
    }

    fn clone_for_task(&self) -> Self {
        Self {
            config: self.config.clone(),
            semaphore: Arc::clone(&self.semaphore),
            running_tasks: Arc::clone(&self.running_tasks),
            progress_sender: self.progress_sender.clone(),
        }
    }

    fn build_dependency_graph<T: ExecutableTask>(
        &self,
        tasks: &[T],
    ) -> TsrcResult<HashMap<String, Vec<String>>> {
        let mut graph = HashMap::new();
        
        for task in tasks {
            graph.insert(task.id().to_string(), task.dependencies());
        }
        
        // 循環依存をチェック
        self.check_circular_dependencies(&graph)?;
        
        Ok(graph)
    }

    fn check_circular_dependencies(
        &self,
        graph: &HashMap<String, Vec<String>>,
    ) -> TsrcResult<()> {
        let mut visited = std::collections::HashSet::new();
        let mut rec_stack = std::collections::HashSet::new();

        for node in graph.keys() {
            if !visited.contains(node) {
                if self.has_cycle(node, graph, &mut visited, &mut rec_stack) {
                    return Err(TsrcError::validation_error(
                        "dependencies",
                        "Circular dependency detected",
                        Some(node.clone()),
                    ));
                }
            }
        }

        Ok(())
    }

    fn has_cycle(
        &self,
        node: &str,
        graph: &HashMap<String, Vec<String>>,
        visited: &mut std::collections::HashSet<String>,
        rec_stack: &mut std::collections::HashSet<String>,
    ) -> bool {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());

        if let Some(dependencies) = graph.get(node) {
            for dep in dependencies {
                if !visited.contains(dep) {
                    if self.has_cycle(dep, graph, visited, rec_stack) {
                        return true;
                    }
                } else if rec_stack.contains(dep) {
                    return true;
                }
            }
        }

        rec_stack.remove(node);
        false
    }

    fn topological_sort(
        &self,
        graph: &HashMap<String, Vec<String>>,
    ) -> TsrcResult<Vec<String>> {
        let mut in_degree = HashMap::new();
        let mut adj_list = HashMap::new();

        // 初期化
        for node in graph.keys() {
            in_degree.insert(node.clone(), 0);
            adj_list.insert(node.clone(), Vec::new());
        }

        // グラフ構築（逆向き）
        for (node, dependencies) in graph {
            for dep in dependencies {
                if !graph.contains_key(dep) {
                    return Err(TsrcError::validation_error(
                        "dependencies",
                        "Unknown dependency",
                        Some(dep.clone()),
                    ));
                }
                adj_list.get_mut(dep).unwrap().push(node.clone());
                *in_degree.get_mut(node).unwrap() += 1;
            }
        }

        // トポロジカルソート
        let mut queue = std::collections::VecDeque::new();
        let mut result = Vec::new();

        // 入次数が0のノードをキューに追加
        for (node, degree) in &in_degree {
            if *degree == 0 {
                queue.push_back(node.clone());
            }
        }

        while let Some(node) = queue.pop_front() {
            result.push(node.clone());

            for neighbor in adj_list.get(&node).unwrap() {
                let degree = in_degree.get_mut(neighbor).unwrap();
                *degree -= 1;
                if *degree == 0 {
                    queue.push_back(neighbor.clone());
                }
            }
        }

        if result.len() != graph.len() {
            return Err(TsrcError::internal_error("Failed to resolve dependencies"));
        }

        Ok(result)
    }

    async fn wait_for_dependencies<T: ExecutableTask>(
        &self,
        task: &T,
        completed_tasks: &std::collections::HashSet<String>,
    ) -> TsrcResult<()> {
        let dependencies = task.dependencies();
        
        if dependencies.is_empty() {
            return Ok(());
        }

        let start_time = Instant::now();
        
        loop {
            let all_satisfied = dependencies.iter().all(|dep| completed_tasks.contains(dep));
            
            if all_satisfied {
                return Ok(());
            }

            if start_time.elapsed() > self.config.dependency_timeout {
                return Err(TsrcError::timeout(self.config.dependency_timeout.as_secs()));
            }

            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    struct TestTask {
        id: String,
        duration: Duration,
        should_fail: bool,
        counter: Arc<AtomicU32>,
    }

    impl TestTask {
        fn new(id: &str, duration: Duration) -> Self {
            Self {
                id: id.to_string(),
                duration,
                should_fail: false,
                counter: Arc::new(AtomicU32::new(0)),
            }
        }

        fn with_failure(mut self) -> Self {
            self.should_fail = true;
            self
        }
    }

    impl Clone for TestTask {
        fn clone(&self) -> Self {
            Self {
                id: self.id.clone(),
                duration: self.duration,
                should_fail: self.should_fail,
                counter: Arc::clone(&self.counter),
            }
        }
    }

    #[async_trait::async_trait]
    impl ExecutableTask for TestTask {
        type Output = u32;

        fn id(&self) -> &str {
            &self.id
        }

        async fn execute(&self, _progress_sender: Option<mpsc::UnboundedSender<TaskProgress>>) -> TsrcResult<Self::Output> {
            tokio::time::sleep(self.duration).await;
            
            if self.should_fail {
                return Err(TsrcError::internal_error("Test task failed"));
            }

            let count = self.counter.fetch_add(1, Ordering::SeqCst);
            Ok(count)
        }

        fn estimated_duration(&self) -> Option<Duration> {
            Some(self.duration)
        }
    }

    #[tokio::test]
    async fn test_single_task_execution() {
        let executor = TaskExecutor::new(ExecutorConfig::default());
        let task = TestTask::new("test-task", Duration::from_millis(100));

        let result = executor.execute_task(task).await.unwrap();

        assert!(result.is_success());
        assert_eq!(result.task_id, "test-task");
        assert!(result.duration >= Duration::from_millis(100));
    }

    #[tokio::test]
    async fn test_failed_task() {
        let executor = TaskExecutor::new(ExecutorConfig::default());
        let task = TestTask::new("failing-task", Duration::from_millis(50)).with_failure();

        let result = executor.execute_task(task).await.unwrap();

        assert!(result.is_failed());
        assert_eq!(result.status, TaskStatus::Failed);
    }

    #[tokio::test]
    async fn test_parallel_execution() {
        let executor = TaskExecutor::new(ExecutorConfig {
            max_concurrent: 2,
            ..ExecutorConfig::default()
        });

        let tasks = vec![
            TestTask::new("task-1", Duration::from_millis(100)),
            TestTask::new("task-2", Duration::from_millis(100)),
            TestTask::new("task-3", Duration::from_millis(100)),
        ];

        let start_time = Instant::now();
        let results = executor.execute_tasks(tasks).await;
        let total_time = start_time.elapsed();

        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|r| r.is_success()));
        
        // 並列実行により、3つのタスクが200ms程度で完了することを確認
        // (2つ並列 + 1つ追加で約200ms)
        assert!(total_time < Duration::from_millis(250));
    }

    #[tokio::test]
    async fn test_task_timeout() {
        let mut config = ExecutorConfig::default();
        config.task_timeout = Some(Duration::from_millis(50));
        
        let executor = TaskExecutor::new(config);
        let task = TestTask::new("slow-task", Duration::from_millis(200));

        let result = executor.execute_task(task).await.unwrap();

        assert_eq!(result.status, TaskStatus::Timeout);
        assert!(result.is_failed());
    }

    #[tokio::test]
    async fn test_task_cancellation() {
        let executor = TaskExecutor::new(ExecutorConfig::default());
        let task = TestTask::new("long-task", Duration::from_secs(10));

        let task_handle = tokio::spawn({
            let executor = executor.clone_for_task();
            async move {
                executor.execute_task(task).await
            }
        });

        // 短時間待ってからキャンセル
        tokio::time::sleep(Duration::from_millis(50)).await;
        executor.cancel_task("long-task").await;

        let result = task_handle.await.unwrap().unwrap();
        assert_eq!(result.status, TaskStatus::Cancelled);
    }
}