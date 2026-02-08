use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Condvar, Mutex};
use std::task::{Context, Poll, Waker};
use std::thread;

/// 线程池任务，包含执行函数和结果发送器
type Task = Box<dyn FnOnce() + Send>;

/// 线程池执行结果
pub struct TaskResult<T> {
    receiver: Arc<Mutex<Option<T>>>,
    waker: Arc<Mutex<Option<Waker>>>,
}

impl<T: Send> Future for TaskResult<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut result = self.receiver.lock().expect("Result lock should not be poisoned");
        if let Some(value) = result.take() {
            Poll::Ready(value)
        } else {
            let mut waker = self.waker.lock().expect("Waker lock should not be poisoned");
            *waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

/// A thread pool for executing tasks concurrently
pub struct ThreadPool {
    workers: Vec<Worker>,
    tasks: Arc<Mutex<VecDeque<Task>>>,
    notifier: Arc<Condvar>,
    shutdown: Arc<Mutex<bool>>,
}

impl ThreadPool {
    pub fn new(size: usize) -> Self {
        assert!(size > 0, "Thread pool size must be greater than zero");

        let tasks = Arc::new(Mutex::new(VecDeque::new()));
        let notifier = Arc::new(Condvar::new());
        let shutdown = Arc::new(Mutex::new(false));

        let workers = (0..size)
            .map(|id| Worker::new(id, Arc::clone(&tasks), Arc::clone(&notifier), Arc::clone(&shutdown)))
            .collect();

        Self {
            workers,
            tasks,
            notifier,
            shutdown,
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let mut tasks = self.tasks.lock().expect("Thread pool tasks lock should not be poisoned");
        tasks.push_back(Box::new(f));
        self.notifier.notify_one();
    }

    /// 执行异步任务并返回Future
    ///
    /// 参考nebula-graph的runMultiJobs模式，支持Scatter-Gather并行计算
    pub fn spawn<F, T>(&self, f: F) -> TaskResult<T>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        let result: Arc<Mutex<Option<T>>> = Arc::new(Mutex::new(None));
        let waker: Arc<Mutex<Option<Waker>>> = Arc::new(Mutex::new(None));

        let result_clone = Arc::clone(&result);
        let waker_clone = Arc::clone(&waker);

        self.execute(move || {
            let value = f();
            let mut res = result_clone.lock().expect("Result lock should not be poisoned");
            *res = Some(value);

            if let Some(w) = waker_clone.lock().expect("Waker lock should not be poisoned").take() {
                w.wake();
            }
        });

        TaskResult { receiver: result, waker }
    }

    /// 并行执行多个任务（Scatter-Gather模式）
    /// 
    /// 参考nebula-graph的Executor::runMultiJobs实现
    /// - scatter: 将数据分批处理
    /// - gather: 收集所有结果
    /// 
    /// # 示例
    /// ```rust
    /// let pool = ThreadPool::new(4);
    /// let data: Vec<i32> = (0..100).collect();
    /// let results = pool.run_multi_jobs(
    ///     |batch| batch.iter().map(|x| x * 2).collect::<Vec<i32>>(),
    ///     data,
    ///     10, // batch_size
    /// ).await;
    /// ```
    pub async fn run_multi_jobs<T, R, F>(
        &self,
        scatter: F,
        data: Vec<T>,
        batch_size: usize,
    ) -> Vec<R>
    where
        T: Send + Clone + 'static,
        R: Send + 'static,
        F: Fn(Vec<T>) -> R + Send + Sync + 'static,
    {
        if data.is_empty() {
            return Vec::new();
        }

        let scatter = Arc::new(scatter);
        let chunks: Vec<Vec<T>> = data
            .chunks(batch_size.max(1))
            .map(|chunk| chunk.iter().cloned().collect())
            .collect();

        let futures: Vec<_> = chunks
            .into_iter()
            .map(|chunk| {
                let scatter = Arc::clone(&scatter);
                self.spawn(move || scatter(chunk))
            })
            .collect();

        let mut results = Vec::new();
        for future in futures {
            results.push(future.await);
        }

        results
    }

    /// 计算批处理大小
    /// 
    /// 参考nebula-graph的getBatchSize实现
    pub fn calculate_batch_size(&self, total_size: usize) -> usize {
        let num_threads = self.workers.len();
        if num_threads == 0 {
            return total_size;
        }
        (total_size + num_threads - 1) / num_threads
    }

    pub fn len(&self) -> usize {
        self.workers.len()
    }

    pub fn is_empty(&self) -> bool {
        self.workers.is_empty()
    }

    pub fn shutdown(&self) {
        {
            let mut shutdown = self.shutdown.lock().expect("Shutdown lock should not be poisoned");
            *shutdown = true;
        }
        self.notifier.notify_all();
    }

    pub fn wait_for_completion(&mut self) {
        for worker in &mut self.workers {
            worker.wait();
        }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        self.shutdown();
        self.wait_for_completion();
    }
}

struct Worker {
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(
        _id: usize,
        tasks: Arc<Mutex<VecDeque<Task>>>,
        notifier: Arc<Condvar>,
        shutdown: Arc<Mutex<bool>>,
    ) -> Self {
        let thread = thread::spawn(move || {
            loop {
                let task = {
                    let mut tasks = tasks.lock().expect("Worker tasks lock should not be poisoned");

                    let is_shutdown = *shutdown.lock().expect("Shutdown lock should not be poisoned");

                    if let Some(task) = tasks.pop_front() {
                        Some(task)
                    } else if is_shutdown {
                        break;
                    } else {
                        loop {
                            tasks = notifier.wait(tasks).expect("Condition variable wait should not fail");

                            if let Some(task) = tasks.pop_front() {
                                break Some(task);
                            }

                            let new_shutdown = *shutdown.lock().expect("Shutdown lock should not be poisoned");
                            if new_shutdown {
                                break None;
                            }
                        }
                    }
                };

                if let Some(task) = task {
                    task();
                }
            }
        });

        Self { thread: Some(thread) }
    }

    fn wait(&mut self) {
        if let Some(handle) = self.thread.take() {
            let _ = handle.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_thread_pool() {
        let pool = ThreadPool::new(4);
        let counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));

        for _ in 0..8 {
            let counter = Arc::clone(&counter);
            pool.execute(move || {
                counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            });
        }

        pool.shutdown();
        drop(pool);
        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 8);
    }

    #[tokio::test]
    async fn test_spawn() {
        let pool = ThreadPool::new(4);
        
        let result = pool.spawn(|| 42).await;
        assert_eq!(result, 42);

        let result = pool.spawn(|| "hello".to_string()).await;
        assert_eq!(result, "hello");
    }

    #[tokio::test]
    async fn test_run_multi_jobs() {
        let pool = ThreadPool::new(4);
        let data: Vec<i32> = (0..100).collect();
        
        let results = pool.run_multi_jobs(
            |batch: Vec<i32>| batch.iter().map(|x| x * 2).collect::<Vec<i32>>(),
            data,
            10,
        ).await;

        let flattened: Vec<i32> = results.into_iter().flatten().collect();
        assert_eq!(flattened.len(), 100);
        assert_eq!(flattened[0], 0);
        assert_eq!(flattened[50], 100);
        assert_eq!(flattened[99], 198);
    }

    #[test]
    fn test_calculate_batch_size() {
        let pool = ThreadPool::new(4);
        
        assert_eq!(pool.calculate_batch_size(100), 25);
        assert_eq!(pool.calculate_batch_size(99), 25);
        assert_eq!(pool.calculate_batch_size(4), 1);
        assert_eq!(pool.calculate_batch_size(3), 1);
    }
}
