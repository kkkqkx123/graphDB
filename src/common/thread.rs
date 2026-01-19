use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tokio::sync::Notify;

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

        std::thread::sleep(Duration::from_millis(100));
        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 8);
    }
}
