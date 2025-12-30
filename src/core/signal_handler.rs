//! Signal handling system for GraphDB
//!
//! This module provides signal handling similar to NebulaGraph's SignalHandler,
//! handling POSIX signals in a daemon environment.

#[cfg(unix)]
use signal_hook::{
    consts::SIGHUP, consts::SIGINT, consts::SIGPIPE, consts::SIGQUIT, consts::SIGTERM,
    iterator::Signals,
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Static instance of the signal handler
// static mut SIGNAL_HANDLER: Option<Arc<SignalHandler>> = None; // 注释掉未使用的静态变量
// static INIT: Once = Once::new(); // 注释掉未使用的静态变量

/// Holds information about a received signal
#[derive(Debug, Clone)]
pub struct SignalInfo {
    pub signal: i32,
    pub pid: Option<u32>, // Process ID if available
    pub uid: Option<u32>, // User ID if available
}

impl SignalInfo {
    pub fn new(signal: i32) -> Self {
        SignalInfo {
            signal,
            pid: None,
            uid: None,
        }
    }

    pub fn to_string(&self) -> String {
        format!("Signal {} received", self.signal)
    }
}

impl std::fmt::Display for SignalInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

/// Signal handling functionality
pub struct SignalHandler {
    shutdown_requested: Arc<AtomicBool>,
    #[cfg(unix)]
    signals: Arc<Signals>,
    #[cfg(not(unix))]
    signals: Arc<std::sync::Mutex<std::collections::HashMap<i32, Box<dyn Fn() + Send + Sync>>>>,

    signal_info: Arc<std::sync::Mutex<Option<SignalInfo>>>,
}

#[cfg(unix)]
impl SignalHandler {
    /// Initialize the signal handler
    pub fn init() -> Result<Arc<Self>, Box<dyn std::error::Error>> {
        let signals = Signals::new(&[SIGTERM, SIGINT, SIGQUIT, SIGHUP, SIGPIPE])?;
        let signals = Arc::new(signals);

        let handler = Arc::new(SignalHandler {
            shutdown_requested: Arc::new(AtomicBool::new(false)),
            signals: signals.clone(),
            signal_info: Arc::new(std::sync::Mutex::new(None)),
        });

        // Spawn a thread to handle signals
        let h = handler.clone();
        std::thread::spawn(move || {
            for sig in signals.forever() {
                h.handle_signal(sig);
            }
        });

        // Store the handler globally
        unsafe {
            INIT.call_once(|| {
                SIGNAL_HANDLER = Some(handler.clone());
            });
        }

        Ok(handler)
    }

    /// Get the global signal handler instance
    pub fn get() -> Option<Arc<SignalHandler>> {
        unsafe { SIGNAL_HANDLER.cloned() }
    }

    /// Handle a received signal
    fn handle_signal(&self, sig: i32) {
        // Update the signal info
        let signal_info = SignalInfo::new(sig);
        match self.signal_info.lock() {
            Ok(mut info) => *info = Some(signal_info.clone()),
            Err(poisoned) => {
                // 尝试从污染的锁中恢复数据
                log::warn!("Signal info lock is poisoned, attempting recovery");
                *poisoned.into_inner() = Some(signal_info.clone());
            }
        }

        match sig {
            SIGTERM | SIGINT | SIGQUIT => {
                eprintln!("Received shutdown signal: {}", sig);
                self.shutdown_requested.store(true, Ordering::Relaxed);
            }
            SIGHUP => {
                eprintln!("Received SIGHUP, might reload configuration");
                // In a real implementation, we might reload configuration here
            }
            SIGPIPE => {
                eprintln!("Received SIGPIPE, ignoring");
                // Ignoring SIGPIPE as is common practice
            }
            _ => {
                eprintln!("Received unhandled signal: {}", sig);
            }
        }
    }

    /// Check if a shutdown signal has been received
    pub fn shutdown_requested(&self) -> bool {
        self.shutdown_requested.load(Ordering::Relaxed)
    }

    /// Install a custom signal handler for a specific signal
    pub fn install<F>(sig: i32, handler: F) -> Result<(), Box<dyn std::error::Error>>
    where
        F: Fn() + Send + Sync + 'static,
    {
        // Register the signal for the iterator
        // In a real implementation, this would be more complex
        // We'll use the default signal handling approach above
        drop(sig);
        drop(handler);
        Ok(())
    }

    /// Install a custom signal handler for multiple signals
    pub fn install_multi<F>(sigs: Vec<i32>, handler: F) -> Result<(), Box<dyn std::error::Error>>
    where
        F: Fn(SignalInfo) + Send + Sync + 'static,
    {
        drop(sigs);
        drop(handler);
        // This would be implemented with more complex logic in production code
        Ok(())
    }

    /// Get the last received signal info
    pub fn get_last_signal(&self) -> Option<SignalInfo> {
        match self.signal_info.lock() {
            Ok(info) => info.clone(),
            Err(poisoned) => {
                // 尝试从污染的锁中恢复数据
                log::warn!(
                    "Signal info lock is poisoned when getting last signal, attempting recovery"
                );
                poisoned.into_inner().clone()
            }
        }
    }
}

#[cfg(unix)]
#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    #[test]
    fn test_signal_handler_creation() {
        let result = SignalHandler::init();
        assert!(result.is_ok());

        let handler = result.expect("SignalHandler::init() should return Ok value");
        assert!(!handler.shutdown_requested());

        // Test getting the global instance
        let global_handler = SignalHandler::get();
        assert!(global_handler.is_some());
    }

    #[test]
    fn test_signal_info() {
        let info = SignalInfo::new(SIGTERM);
        assert_eq!(info.signal, SIGTERM);
        assert!(info.to_string().contains("Signal"));
    }
}

// For non-Unix systems, we'll provide a minimal mock implementation
#[cfg(not(unix))]
impl SignalHandler {
    /// Initialize the signal handler (mock for non-Unix)
    pub fn init() -> Result<Arc<Self>, Box<dyn std::error::Error>> {
        Ok(Arc::new(SignalHandler {
            shutdown_requested: Arc::new(AtomicBool::new(false)),
            signals: Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
            signal_info: Arc::new(std::sync::Mutex::new(None)),
        }))
    }

    /// Get the global signal handler instance (mock for non-Unix)
    pub fn get() -> Option<Arc<SignalHandler>> {
        // For non-unix systems, we return a default instance temporarily
        // since we can't use the static storage mechanism
        Some(Arc::new(SignalHandler {
            shutdown_requested: Arc::new(AtomicBool::new(false)),
            signals: Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
            signal_info: Arc::new(std::sync::Mutex::new(None)),
        }))
    }

    /// Check if a shutdown signal has been received (always false for non-Unix)
    pub fn shutdown_requested(&self) -> bool {
        self.shutdown_requested.load(Ordering::Relaxed)
    }

    /// Install a custom signal handler for a specific signal (no-op for non-Unix)
    pub fn install<F>(_: i32, _: F) -> Result<(), Box<dyn std::error::Error>>
    where
        F: Fn() + Send + Sync + 'static,
    {
        Ok(())
    }

    /// Install a custom signal handler for multiple signals (no-op for non-Unix)
    pub fn install_multi<F>(_: Vec<i32>, _: F) -> Result<(), Box<dyn std::error::Error>>
    where
        F: Fn(SignalInfo) + Send + Sync + 'static,
    {
        Ok(())
    }

    /// Get the last received signal info (always None for non-Unix)
    pub fn get_last_signal(&self) -> Option<SignalInfo> {
        None
    }
}
