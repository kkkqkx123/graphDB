use std::collections::HashMap;
use std::env;
use std::process;

/// Represents a process identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ProcessId(u32);

impl ProcessId {
    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

impl From<u32> for ProcessId {
    fn from(id: u32) -> Self {
        ProcessId(id)
    }
}

/// Information about a running process
#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: ProcessId,
    pub name: String,
    pub cmd: String,
    pub cwd: String,
    pub root: String,
    pub start_time: std::time::SystemTime,
    pub parent_pid: Option<ProcessId>,
    pub status: ProcessStatus,
    pub memory_usage: u64,
    pub cpu_usage: f64,
    pub open_files: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum ProcessStatus {
    Running,
    Sleeping,
    Stopped,
    Zombie,
    Unknown,
}

/// Process management utilities
pub struct ProcessManager {
    monitored_processes: std::sync::Arc<std::sync::Mutex<HashMap<ProcessId, ProcessInfo>>>,
}

impl ProcessManager {
    pub fn new() -> Self {
        Self {
            monitored_processes: std::sync::Arc::new(std::sync::Mutex::new(HashMap::new())),
        }
    }

    /// Get the current process ID
    pub fn current_pid(&self) -> ProcessId {
        ProcessId(process::id())
    }

    /// Get information about the current process
    pub fn current_process_info(
        &self,
    ) -> Result<ProcessInfo, Box<dyn std::error::Error + Send + Sync>> {
        let pid = ProcessId(process::id());

        Ok(ProcessInfo {
            pid,
            name: {
                let exe_path = env::current_exe()?;
                let opt_name = exe_path.file_name();
                match opt_name {
                    Some(name) => name.to_string_lossy().to_string(),
                    None => String::default(),
                }
            },
            cmd: env::args().collect::<Vec<_>>().join(" "),
            cwd: env::current_dir()?.to_string_lossy().to_string(),
            root: "/".to_string(),
            start_time: std::time::SystemTime::now(),
            parent_pid: None,
            status: ProcessStatus::Running,
            memory_usage: get_memory_usage()?,
            cpu_usage: 0.0,
            open_files: vec![],
        })
    }

    /// Get information about a specific process by PID
    pub fn get_process_info(&self, pid: ProcessId) -> Option<ProcessInfo> {
        if pid.as_u32() == process::id() {
            return self.current_process_info().ok();
        }

        self.monitored_processes
            .lock()
            .expect("Process monitor lock should not be poisoned")
            .get(&pid)
            .cloned()
    }

    /// Check if a process is running
    pub fn is_process_running(&self, pid: ProcessId) -> bool {
        if pid.as_u32() == process::id() {
            return true;
        }

        self.monitored_processes
            .lock()
            .expect("Process monitor lock should not be poisoned")
            .contains_key(&pid)
    }

    /// List all processes (simplified implementation)
    pub fn list_processes(
        &self,
    ) -> Result<Vec<ProcessInfo>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(vec![self.current_process_info()?])
    }

    /// Register a process to be monitored
    pub fn register_process(&self, info: ProcessInfo) {
        self.monitored_processes
            .lock()
            .expect("Process monitor lock should not be poisoned")
            .insert(info.pid, info);
    }

    /// Unregister a process from monitoring
    pub fn unregister_process(&self, pid: ProcessId) -> Option<ProcessInfo> {
        self.monitored_processes
            .lock()
            .expect("Process monitor lock should not be poisoned")
            .remove(&pid)
    }

    /// Send a signal to a process
    pub fn send_signal(
        &self,
        pid: ProcessId,
        _signal: i32,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if pid.as_u32() == process::id() {
            println!("Signal {} to current process", _signal);
            Ok(())
        } else {
            Err(format!(
                "Cannot send signal to process {} in simplified implementation",
                pid.as_u32()
            )
            .into())
        }
    }

    /// Wait for a process to finish (simplified implementation)
    pub fn wait_for_process(
        &self,
        pid: ProcessId,
        timeout: Option<std::time::Duration>,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        if self.is_process_running(pid) {
            if let Some(timeout) = timeout {
                std::thread::sleep(timeout);
                Ok(false)
            } else {
                Err("No timeout specified in simplified implementation".into())
            }
        } else {
            Ok(true)
        }
    }
}

/// Get memory usage for the current process
#[cfg(unix)]
fn get_memory_usage() -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    let file = File::open("/proc/self/status")?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line?;
        if line.starts_with("VmRSS:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let kb: u64 = parts[1].parse()?;
                return Ok(kb * 1024);
            }
        }
    }

    Ok(0)
}

/// Get memory usage for the current process (Windows)
#[cfg(windows)]
fn get_memory_usage() -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
    use std::mem;
    use winapi::um::processthreadsapi::GetCurrentProcess;
    use winapi::um::psapi::GetProcessMemoryInfo;
    use winapi::um::psapi::PROCESS_MEMORY_COUNTERS;

    let handle = unsafe { GetCurrentProcess() };
    let mut pmc: PROCESS_MEMORY_COUNTERS = unsafe { mem::zeroed() };
    pmc.cb = mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32;

    let result = unsafe {
        GetProcessMemoryInfo(
            handle,
            &mut pmc as *mut PROCESS_MEMORY_COUNTERS,
            mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
        )
    };

    if result != 0 {
        Ok(pmc.WorkingSetSize as u64)
    } else {
        Ok(0)
    }
}

/// Get memory usage for the current process (fallback for other platforms)
#[cfg(not(any(unix, windows)))]
fn get_memory_usage() -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
    Ok(0)
}

/// Get the current process environment
pub fn get_environment() -> HashMap<String, String> {
    std::env::vars().collect()
}

/// Get a specific environment variable
pub fn get_environment_var(key: &str) -> Option<String> {
    std::env::var(key).ok()
}

/// Set an environment variable for the current process
pub fn set_environment_var(key: &str, value: &str) {
    std::env::set_var(key, value);
}

/// Initialize process management
pub fn init_process_management() -> ProcessManager {
    ProcessManager::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_id() {
        let pid = ProcessId(1234);
        assert_eq!(pid.as_u32(), 1234);
    }

    #[test]
    fn test_current_process_info() {
        let pm = ProcessManager::new();
        let info = pm
            .current_process_info()
            .expect("Current process info should be available");

        assert_eq!(info.pid.as_u32(), process::id());
        assert!(!info.name.is_empty());
    }

    #[test]
    fn test_process_manager() {
        let pm = ProcessManager::new();
        let pid = pm.current_pid();

        assert!(pm.is_process_running(pid));
        assert!(pm.get_process_info(pid).is_some());
    }

    #[test]
    fn test_environment_functions() {
        set_environment_var("TEST_VAR", "test_value");
        assert_eq!(
            get_environment_var("TEST_VAR"),
            Some("test_value".to_string())
        );

        let env = get_environment();
        assert!(env.contains_key("TEST_VAR"));
    }
}
