use graphdb::common::process::*;
use std::process;

#[cfg(unix)]
use signal_hook::{consts::SIGINT, consts::SIGTERM};

#[test]
fn test_process_id_new() {
    let pid = ProcessId::from(1234);
    assert_eq!(pid.as_u32(), 1234);
}

#[test]
fn test_process_id_from_u32() {
    let pid = ProcessId::from(5678u32);
    assert_eq!(pid.as_u32(), 5678);
}

#[test]
fn test_process_id_partial_eq() {
    let pid1 = ProcessId::from(1000);
    let pid2 = ProcessId::from(1000);
    let pid3 = ProcessId::from(2000);

    assert_eq!(pid1, pid2);
    assert_ne!(pid1, pid3);
}

#[test]
fn test_process_id_hash() {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let pid1 = ProcessId::from(1000);
    let pid2 = ProcessId::from(1000);
    let pid3 = ProcessId::from(2000);

    let mut hasher1 = DefaultHasher::new();
    let mut hasher2 = DefaultHasher::new();
    let mut hasher3 = DefaultHasher::new();

    pid1.hash(&mut hasher1);
    pid2.hash(&mut hasher2);
    pid3.hash(&mut hasher3);

    assert_eq!(hasher1.finish(), hasher2.finish());
    assert_ne!(hasher1.finish(), hasher3.finish());
}

#[test]
fn test_process_status_variants() {
    let _running = ProcessStatus::Running;
    let _sleeping = ProcessStatus::Sleeping;
    let _stopped = ProcessStatus::Stopped;
    let _zombie = ProcessStatus::Zombie;
    let _unknown = ProcessStatus::Unknown;
}

#[test]
fn test_process_manager_new() {
    let pm = ProcessManager::new();
    let _pid = pm.current_pid();
}

#[test]
fn test_process_manager_current_pid() {
    let pm = ProcessManager::new();
    let pid = pm.current_pid();
    assert_eq!(pid.as_u32(), process::id());
}

#[test]
fn test_process_manager_current_process_info() {
    let pm = ProcessManager::new();
    let info = pm.current_process_info().expect("Should get process info");

    assert_eq!(info.pid.as_u32(), process::id());
    assert!(!info.name.is_empty());
    assert!(!info.cmd.is_empty());
    assert!(!info.cwd.is_empty());
    assert!(matches!(info.status, ProcessStatus::Running));
}

#[test]
fn test_process_manager_is_process_running_current() {
    let pm = ProcessManager::new();
    let pid = pm.current_pid();

    assert!(pm.is_process_running(pid));
}

#[test]
fn test_process_manager_is_process_running_other() {
    let pm = ProcessManager::new();
    let fake_pid = ProcessId::from(999999);

    assert!(!pm.is_process_running(fake_pid));
}

#[test]
fn test_process_manager_get_process_info_current() {
    let pm = ProcessManager::new();
    let pid = pm.current_pid();
    let info = pm.get_process_info(pid);

    assert!(info.is_some());
    assert_eq!(info.unwrap().pid, pid);
}

#[test]
fn test_process_manager_get_process_info_other() {
    let pm = ProcessManager::new();
    let fake_pid = ProcessId::from(999999);
    let info = pm.get_process_info(fake_pid);

    assert!(info.is_none());
}

#[test]
fn test_process_manager_list_processes() {
    let pm = ProcessManager::new();
    let processes = pm.list_processes().expect("Should list processes");

    assert!(!processes.is_empty());
    assert_eq!(processes[0].pid.as_u32(), process::id());
}

#[test]
fn test_process_manager_register_process() {
    let pm = ProcessManager::new();
    let info = pm.current_process_info().expect("Should get process info");

    pm.register_process(info.clone());

    let retrieved = pm.get_process_info(info.pid);
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().pid, info.pid);
}

#[test]
fn test_process_manager_unregister_process() {
    let pm = ProcessManager::new();

    let fake_info = ProcessInfo {
        pid: ProcessId::from(999999),
        name: "fake_process".to_string(),
        cmd: "fake_command".to_string(),
        cwd: "/fake".to_string(),
        root: "/fake".to_string(),
        start_time: std::time::SystemTime::UNIX_EPOCH,
        parent_pid: None,
        status: ProcessStatus::Running,
        memory_usage: 0,
        cpu_usage: 0.0,
        open_files: vec![],
    };

    pm.register_process(fake_info.clone());
    let removed = pm.unregister_process(fake_info.pid);

    assert!(removed.is_some());
    assert_eq!(removed.unwrap().pid, fake_info.pid);

    let after_remove = pm.get_process_info(fake_info.pid);
    assert!(after_remove.is_none());
}

#[test]
fn test_process_manager_send_signal_current() {
    let pm = ProcessManager::new();
    let pid = pm.current_pid();

    let result = pm.send_signal(pid, 15);
    assert!(result.is_ok());
}

#[test]
fn test_process_manager_send_signal_other() {
    let pm = ProcessManager::new();
    let fake_pid = ProcessId::from(999999);

    let result = pm.send_signal(fake_pid, 15);
    assert!(result.is_err());
}

#[test]
fn test_process_manager_wait_for_process_timeout() {
    let pm = ProcessManager::new();
    let fake_pid = ProcessId::from(999999);

    let result = pm.wait_for_process(fake_pid, Some(std::time::Duration::from_millis(10)));
    assert!(result.is_ok());
    assert!(result.unwrap());
}

#[test]
fn test_process_manager_wait_for_process_no_timeout() {
    let pm = ProcessManager::new();

    let fake_info = ProcessInfo {
        pid: ProcessId::from(999998),
        name: "fake_process".to_string(),
        cmd: "fake_command".to_string(),
        cwd: "/fake".to_string(),
        root: "/fake".to_string(),
        start_time: std::time::SystemTime::UNIX_EPOCH,
        parent_pid: None,
        status: ProcessStatus::Running,
        memory_usage: 0,
        cpu_usage: 0.0,
        open_files: vec![],
    };

    pm.register_process(fake_info.clone());

    let result = pm.wait_for_process(fake_info.pid, None);
    assert!(result.is_err());
}

#[cfg(unix)]
#[test]
fn test_signal_handler_new() {
    let handler = SignalHandler::new(&[SIGINT, SIGTERM]);
    assert!(handler.is_ok());
}

#[cfg(unix)]
#[test]
fn test_signal_handler_register_handler() {
    let handler = SignalHandler::new(&[SIGINT, SIGTERM]).expect("Should create handler");
    let called = std::sync::atomic::AtomicBool::new(false);

    let result = handler.register_handler(SIGINT, move || {
        called.store(true, std::sync::atomic::Ordering::SeqCst);
    });

    assert!(result.is_ok());
}

#[cfg(unix)]
#[test]
fn test_signal_handler_start_handling() {
    let handler = SignalHandler::new(&[]).expect("Should create handler");
    let result = handler.start_handling();
    assert!(result.is_ok());
}

#[test]
fn test_get_environment() {
    let env = get_environment();
    assert!(!env.is_empty());
    let has_path = env.contains_key("PATH") || env.contains_key("Path");
    assert!(has_path, "Environment should contain PATH or Path variable");
}

#[test]
fn test_get_environment_var() {
    std::env::set_var("TEST_PROCESS_VAR", "test_value");

    let value = get_environment_var("TEST_PROCESS_VAR");
    assert_eq!(value, Some("test_value".to_string()));

    std::env::remove_var("TEST_PROCESS_VAR");
}

#[test]
fn test_set_environment_var() {
    set_environment_var("TEST_SET_VAR", "set_value");

    let value = std::env::var("TEST_SET_VAR");
    assert_eq!(value, Ok("set_value".to_string()));

    std::env::remove_var("TEST_SET_VAR");
}

#[test]
fn test_init_process_management() {
    let pm = init_process_management();
    let pid = pm.current_pid();
    assert!(pm.is_process_running(pid));
}

#[test]
fn test_process_info_clone() {
    let pm = ProcessManager::new();
    let info = pm.current_process_info().expect("Should get process info");

    let cloned = info.clone();
    assert_eq!(cloned.pid, info.pid);
    assert_eq!(cloned.name, info.name);
}

#[test]
fn test_process_status_clone() {
    let status = ProcessStatus::Running;
    let cloned = status.clone();
    match cloned {
        ProcessStatus::Running => {}
        _ => panic!("Expected Running status"),
    }
}

#[test]
fn test_system_resources_get_disk_usage() {
    let usage = system_resources::get_disk_usage(".");
    assert!(usage.is_ok());
    let size = usage.unwrap();
    assert!(size > 0);
}

#[test]
fn test_system_resources_is_path_accessible_existing() {
    let accessible = system_resources::is_path_accessible(".");
    assert!(accessible);
}

#[test]
fn test_system_resources_is_path_accessible_nonexistent() {
    let accessible = system_resources::is_path_accessible("/nonexistent_path_12345");
    assert!(!accessible);
}

#[test]
fn test_system_resource_usage_get() {
    let usage = system_resources::get_system_usage();
    assert!(usage.is_ok());

    let u = usage.unwrap();
    assert!(u.cpu_count > 0);
}

#[test]
fn test_process_execution_command_exists() {
    let exists = process_execution::command_exists("cargo");
    assert!(exists, "cargo should exist in PATH for testing");
}

#[test]
fn test_process_execution_command_not_exists() {
    let exists = process_execution::command_exists("this_command_definitely_does_not_exist_12345");
    assert!(!exists);
}

#[test]
fn test_process_execution_execute_command() {
    let output = process_execution::execute_command("cargo", &["--version"]);
    assert!(output.is_ok());
    let result = output.unwrap();
    assert!(result.contains("cargo"));
}

#[test]
fn test_process_execution_execute_command_failure() {
    let output = process_execution::execute_command("nonexistent_command_12345", &[]);
    assert!(output.is_err());
}
