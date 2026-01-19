use std::collections::VecDeque;
use std::future::Future;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex, RwLock};
use std::thread;
use std::time::Duration;
use tokio::sync::Notify;
use tokio::task;

/// A thread pool for executing tasks concurrently
pub struct ThreadPool {
    workers: Vec<Worker>,
    tasks: Arc<Mutex<VecDeque<Box<dyn FnOnce() + Send>>>>,
    notifier: Arc<Notify>,
    shutdown: Arc<Mutex<bool>>,
}

impl ThreadPool {
    pub fn new(size: usize) -> Self {
        assert!(size > 0, "Thread pool size must be greater than zero");

        let mut workers = Vec::with_capacity(size);
        let tasks = Arc::new(Mutex::new(VecDeque::new()));
        let notifier = Arc::new(Notify::new());
        let shutdown = Arc::new(Mutex::new(false));

        for id in 0..size {
            workers.push(Worker::new(
                id,
                Arc::clone(&tasks),
                Arc::clone(&notifier),
                Arc::clone(&shutdown),
            ));
        }

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
        {
            let mut tasks = self
                .tasks
                .lock()
                .expect("Thread pool tasks lock should not be poisoned");
            tasks.push_back(Box::new(f));
        }

        self.notifier.notify_one();
    }

    pub fn len(&self) -> usize {
        self.workers.len()
    }

    pub fn shutdown(&self) {
        let mut shutdown = self.shutdown.lock().expect("Shutdown lock should not be poisoned");
        *shutdown = true;
        drop(shutdown);

        self.notifier.notify_waiters();
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
    _thread: Option<thread::JoinHandle<()>>,
    _notifier: Arc<Notify>,
}

impl Worker {
    fn new(
        id: usize,
        tasks: Arc<Mutex<VecDeque<Box<dyn FnOnce() + Send>>>>,
        notifier: Arc<Notify>,
        shutdown: Arc<Mutex<bool>>,
    ) -> Self {
        let _notifier = Arc::clone(&notifier);
        let _thread = thread::spawn(move || {
            loop {
                let task = {
                    let mut tasks = tasks.lock().expect("Worker tasks lock should not be poisoned");
                    tasks.pop_front()
                };

                if let Some(task) = task {
                    task();
                } else {
                    let should_shutdown = {
                        let shutdown = shutdown.lock().expect("Shutdown lock should not be poisoned");
                        *shutdown
                    };

                    if should_shutdown {
                        break;
                    }

                    std::thread::sleep(Duration::from_millis(10));
                }
            }
        });

        Self { _thread: Some(_thread), _notifier }
    }

    fn wait(&mut self) {
        if let Some(handle) = self._thread.take() {
            let _ = handle.join();
        }
    }
}

/// A thread-safe counter
#[derive(Debug)]
pub struct AtomicCounter {
    value: AtomicUsize,
}

impl AtomicCounter {
    pub fn new(initial: usize) -> Self {
        Self {
            value: AtomicUsize::new(initial),
        }
    }

    pub fn increment(&self) -> usize {
        self.value.fetch_add(1, Ordering::SeqCst)
    }

    pub fn decrement(&self) -> usize {
        self.value.fetch_sub(1, Ordering::SeqCst)
    }

    pub fn get(&self) -> usize {
        self.value.load(Ordering::SeqCst)
    }

    pub fn set(&self, value: usize) {
        self.value.store(value, Ordering::SeqCst);
    }
}

/// A thread-safe lazy initializer using OnceLock
pub struct Lazy<T> {
    value: RwLock<Option<T>>,
    init_fn: Box<dyn Fn() -> T + Send + Sync>,
}

impl<T> Lazy<T> {
    pub fn new<F>(init_fn: F) -> Self
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        Self {
            value: RwLock::new(None),
            init_fn: Box::new(init_fn),
        }
    }

    pub fn get(&self) -> T
    where
        T: Clone,
    {
        {
            if let Ok(guard) = self.value.try_read() {
                if let Some(ref value) = *guard {
                    return value.clone();
                }
            }
        }

        let mut guard = self
            .value
            .write()
            .expect("Lazy value lock should not be poisoned");
        if let Some(ref value) = *guard {
            return value.clone();
        }

        let value = (self.init_fn)();
        *guard = Some(value.clone());
        value
    }
}

/// A condition variable for thread synchronization
#[derive(Debug)]
pub struct ConditionVariable {
    condvar: Condvar,
}

impl ConditionVariable {
    pub fn new() -> Self {
        Self {
            condvar: Condvar::new(),
        }
    }

    pub fn wait<'a>(&self, guard: std::sync::MutexGuard<'a, ()>) -> std::sync::MutexGuard<'a, ()> {
        self.condvar
            .wait(guard)
            .expect("Condition variable should not be corrupted")
    }

    pub fn wait_timeout<'a>(
        &self,
        guard: std::sync::MutexGuard<'a, ()>,
        timeout: Duration,
    ) -> (std::sync::MutexGuard<'a, ()>, std::sync::mpsc::RecvTimeoutError) {
        let (guard, timed_out) = self
            .condvar
            .wait_timeout(guard, timeout)
            .expect("Condition variable should not be corrupted");

        if timed_out.timed_out() {
            (guard, std::sync::mpsc::RecvTimeoutError::Timeout)
        } else {
            (guard, std::sync::mpsc::RecvTimeoutError::Disconnected)
        }
    }

    pub fn notify_one(&self) {
        self.condvar.notify_one();
    }

    pub fn notify_all(&self) {
        self.condvar.notify_all();
    }
}

/// A thread-safe manager for managing multiple threads
pub struct ThreadManager {
    active_threads: Arc<Mutex<Vec<thread::JoinHandle<()>>>>,
}

impl ThreadManager {
    pub fn new() -> Self {
        Self {
            active_threads: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn spawn<F>(&self, f: F) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
    where
        F: FnOnce() + Send + 'static,
    {
        let handle = thread::spawn(f);
        self.active_threads
            .lock()
            .expect("Thread manager active threads lock should not be poisoned")
            .push(handle);
        Ok(())
    }

    pub fn active_count(&self) -> usize {
        let threads = self
            .active_threads
            .lock()
            .expect("Thread manager active threads lock should not be poisoned");
        threads.len()
    }

    pub fn join_all(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut threads = self
            .active_threads
            .lock()
            .expect("Thread manager active threads lock should not be poisoned");
        let mut errors = Vec::new();

        for handle in threads.drain(..) {
            if let Err(e) = handle.join() {
                errors.push(format!("Thread error: {:?}", e));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors.join("; ").into())
        }
    }
}

/// A future-based thread utility for async operations
pub mod async_utils {
    use super::*;

    /// Execute a blocking operation in a way that doesn't block the async runtime
    pub async fn spawn_blocking<F, R>(f: F) -> R
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static,
    {
        task::spawn_blocking(f)
            .await
            .expect("Blocking task should complete successfully")
    }

    /// Run multiple futures concurrently and return all results
    pub async fn join_all<T>(futures: Vec<impl Future<Output = T>>) -> Vec<T> {
        futures::future::join_all(futures).await
    }

    /// Timeout a future with a specified duration
    pub async fn timeout<T>(
        duration: Duration,
        future: impl Future<Output = T>,
    ) -> Result<T, tokio::time::error::Elapsed> {
        tokio::time::timeout(duration, future).await
    }

    /// Create a channel for async communication
    pub fn channel<T>() -> (tokio::sync::mpsc::Sender<T>, tokio::sync::mpsc::Receiver<T>) {
        tokio::sync::mpsc::channel(100)
    }

    /// Create a oneshot channel for single-value async communication
    pub fn oneshot<T>() -> (
        tokio::sync::oneshot::Sender<T>,
        tokio::sync::oneshot::Receiver<T>,
    ) {
        tokio::sync::oneshot::channel()
    }
}

/// Thread-local storage for data that should be local to each thread
#[derive(Debug)]
pub struct ThreadLocal<T: 'static> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T: 'static> ThreadLocal<T> {
    /// Create a new ThreadLocal with a default value
    pub fn new<F>(_init_fn: F) -> Self
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }

    /// Get a reference to value for the current thread
    pub fn with<F, R>(&self, _f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        unimplemented!("Use the thread_local! macro for actual implementation")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_atomic_counter() {
        let counter = AtomicCounter::new(0);
        assert_eq!(counter.get(), 0);

        counter.increment();
        assert_eq!(counter.get(), 1);

        counter.set(10);
        assert_eq!(counter.get(), 10);
    }

    #[test]
    fn test_thread_pool() {
        let pool = ThreadPool::new(4);
        let counter = Arc::new(AtomicCounter::new(0));

        for _ in 0..8 {
            let counter = Arc::clone(&counter);
            pool.execute(move || {
                counter.increment();
            });
        }

        std::thread::sleep(Duration::from_millis(100));
        assert_eq!(counter.get(), 8);
    }

    #[test]
    fn test_condition_variable() {
        let pair = Arc::new((Mutex::new(false), Condvar::new()));
        let pair2 = Arc::clone(&pair);

        let t = thread::spawn(move || {
            let (lock, cvar) = &*pair2;
            let mut started = lock.lock().expect("Test mutex lock should not be poisoned");
            *started = true;
            cvar.notify_one();
        });

        let (lock, cvar) = &*pair;
        let mut started = lock.lock().expect("Test mutex lock should not be poisoned");
        while !*started {
            started = cvar
                .wait(started)
                .expect("Test condition variable should not be corrupted");
        }

        t.join().expect("Test thread should complete successfully");
    }

    #[tokio::test]
    async fn test_async_utils() {
        let result = async_utils::spawn_blocking(|| {
            std::thread::sleep(Duration::from_millis(10));
            42
        })
        .await;

        assert_eq!(result, 42);

        let timeout_result = async_utils::timeout(Duration::from_millis(50), async {
            tokio::time::sleep(Duration::from_millis(10)).await;
            "done"
        })
        .await;

        assert!(timeout_result.is_ok());
        assert_eq!(timeout_result.expect("Timeout result should be ok"), "done");
    }
}
