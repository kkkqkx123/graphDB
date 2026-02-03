use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

/// A thread pool for executing tasks concurrently
pub struct ThreadPool {
    workers: Vec<Worker>,
    tasks: Arc<Mutex<VecDeque<Box<dyn FnOnce() + Send>>>>,
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

    pub fn len(&self) -> usize {
        self.workers.len()
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
        tasks: Arc<Mutex<VecDeque<Box<dyn FnOnce() + Send>>>>,
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
}
