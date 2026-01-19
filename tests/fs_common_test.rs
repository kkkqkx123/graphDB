use graphdb::common::fs::*;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_file_attributes_new() {
    let attrs = FileAttributes {
        size: 1024,
        created: std::time::SystemTime::UNIX_EPOCH,
        modified: std::time::SystemTime::UNIX_EPOCH,
        accessed: std::time::SystemTime::UNIX_EPOCH,
        is_directory: false,
        is_file: true,
        is_symlink: false,
        permissions: 0o644,
    };

    assert_eq!(attrs.size, 1024);
    assert!(attrs.is_file);
    assert!(!attrs.is_directory);
}

#[test]
fn test_file_attributes_serialization() {
    let attrs = FileAttributes {
        size: 1024,
        created: std::time::SystemTime::UNIX_EPOCH,
        modified: std::time::SystemTime::UNIX_EPOCH,
        accessed: std::time::SystemTime::UNIX_EPOCH,
        is_directory: false,
        is_file: true,
        is_symlink: false,
        permissions: 0o644,
    };

    let json = serde_json::to_string(&attrs).expect("Failed to serialize");
    let deserialized: FileAttributes = serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(attrs.size, deserialized.size);
    assert_eq!(attrs.is_file, deserialized.is_file);
}

#[test]
fn test_fs_error_io() {
    let error = FsError::IoError(std::io::Error::new(std::io::ErrorKind::NotFound, "test"));
    let error_msg = error.to_string();
    assert!(error_msg.contains("IO error"));
}

#[test]
fn test_fs_error_path_not_found() {
    let error = FsError::PathNotFound("test_path".to_string());
    let error_msg = error.to_string();
    assert!(error_msg.contains("Path not found"));
}

#[test]
fn test_fs_error_permission_denied() {
    let error = FsError::PermissionDenied("test_path".to_string());
    let error_msg = error.to_string();
    assert!(error_msg.contains("Permission denied"));
}

#[test]
fn test_fs_error_invalid_path() {
    let error = FsError::InvalidPath("test_path".to_string());
    let error_msg = error.to_string();
    assert!(error_msg.contains("Invalid path"));
}

#[test]
fn test_fs_error_already_exists() {
    let error = FsError::AlreadyExists("test_path".to_string());
    let error_msg = error.to_string();
    assert!(error_msg.contains("File already exists"));
}

#[test]
fn test_fs_error_not_a_directory() {
    let error = FsError::NotADirectory("test_path".to_string());
    let error_msg = error.to_string();
    assert!(error_msg.contains("Not a directory"));
}

#[test]
fn test_fs_error_not_a_file() {
    let error = FsError::NotAFile("test_path".to_string());
    let error_msg = error.to_string();
    assert!(error_msg.contains("Not a file"));
}

#[test]
fn test_filesystemutils_exists_true() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let file_path = temp_dir.path().join("test_file.txt");
    fs::write(&file_path, "test content").expect("Failed to write file");

    assert!(FileSystemUtils::exists(&file_path));
}

#[test]
fn test_filesystemutils_exists_false() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let file_path = temp_dir.path().join("nonexistent.txt");

    assert!(!FileSystemUtils::exists(&file_path));
}

#[test]
fn test_filesystemutils_is_file() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let file_path = temp_dir.path().join("test_file.txt");
    fs::write(&file_path, "test content").expect("Failed to write file");

    assert!(FileSystemUtils::is_file(&file_path));
}

#[test]
fn test_filesystemutils_is_directory() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    assert!(FileSystemUtils::is_directory(temp_dir.path()));
}

#[test]
fn test_filesystemutils_get_attributes() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let file_path = temp_dir.path().join("test_file.txt");
    fs::write(&file_path, "test content").expect("Failed to write file");

    let attrs = FileSystemUtils::get_attributes(&file_path).expect("Failed to get attributes");
    assert_eq!(attrs.size, 12);
    assert!(attrs.is_file);
}

#[test]
fn test_filesystemutils_create_dir_all() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let new_dir = temp_dir.path().join("new_dir").join("sub_dir");

    FileSystemUtils::create_dir_all(&new_dir).expect("Failed to create dir");

    assert!(FileSystemUtils::is_directory(&new_dir));
}

#[test]
fn test_filesystemutils_create_file() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let file_path = temp_dir.path().join("new_file.txt");

    FileSystemUtils::create_file(&file_path).expect("Failed to create file");

    assert!(FileSystemUtils::exists(&file_path));
}

#[test]
fn test_filesystemutils_read_to_string() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let file_path = temp_dir.path().join("test_file.txt");
    let content = "Hello, World!";
    fs::write(&file_path, content).expect("Failed to write file");

    let read_content = FileSystemUtils::read_to_string(&file_path).expect("Failed to read");

    assert_eq!(content, read_content);
}

#[test]
fn test_filesystemutils_read() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let file_path = temp_dir.path().join("test_file.txt");
    let content = b"Hello, World!";
    fs::write(&file_path, content).expect("Failed to write file");

    let read_content = FileSystemUtils::read(&file_path).expect("Failed to read");

    assert_eq!(content.as_slice(), read_content.as_slice());
}

#[test]
fn test_filesystemutils_write() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let file_path = temp_dir.path().join("test_file.txt");
    let content = "Hello, World!";

    FileSystemUtils::write(&file_path, content).expect("Failed to write");

    let read_content = FileSystemUtils::read_to_string(&file_path).expect("Failed to read");
    assert_eq!(content, read_content);
}

#[test]
fn test_filesystemutils_append() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let file_path = temp_dir.path().join("test_file.txt");
    fs::write(&file_path, "Hello").expect("Failed to write file");

    FileSystemUtils::append(&file_path, ", World!").expect("Failed to append");

    let content = FileSystemUtils::read_to_string(&file_path).expect("Failed to read");
    assert_eq!(content, "Hello, World!");
}

#[test]
fn test_filesystemutils_copy() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let source_path = temp_dir.path().join("source.txt");
    let dest_path = temp_dir.path().join("dest.txt");
    fs::write(&source_path, "test content").expect("Failed to write file");

    FileSystemUtils::copy(&source_path, &dest_path).expect("Failed to copy");

    let content = FileSystemUtils::read_to_string(&dest_path).expect("Failed to read");
    assert_eq!(content, "test content");
}

#[test]
fn test_filesystemutils_remove_file() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let file_path = temp_dir.path().join("test_file.txt");
    fs::write(&file_path, "test content").expect("Failed to write file");

    FileSystemUtils::remove_file(&file_path).expect("Failed to remove file");

    assert!(!FileSystemUtils::exists(&file_path));
}

#[test]
fn test_filesystemutils_remove_dir() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let dir_path = temp_dir.path().join("empty_dir");
    fs::create_dir(&dir_path).expect("Failed to create dir");

    FileSystemUtils::remove_dir(&dir_path).expect("Failed to remove dir");

    assert!(!FileSystemUtils::exists(&dir_path));
}

#[test]
fn test_filesystemutils_rename() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let old_path = temp_dir.path().join("old_name.txt");
    let new_path = temp_dir.path().join("new_name.txt");
    fs::write(&old_path, "test content").expect("Failed to write file");

    FileSystemUtils::rename(&old_path, &new_path).expect("Failed to rename");

    assert!(!FileSystemUtils::exists(&old_path));
    assert!(FileSystemUtils::exists(&new_path));
}

#[test]
fn test_filesystemutils_read_dir() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let file1 = temp_dir.path().join("file1.txt");
    let file2 = temp_dir.path().join("file2.txt");
    fs::write(&file1, "content1").expect("Failed to write file");
    fs::write(&file2, "content2").expect("Failed to write file");

    let entries = FileSystemUtils::read_dir(temp_dir.path()).expect("Failed to read dir");

    assert_eq!(entries.len(), 2);
}

#[test]
fn test_filesystemutils_walk_dir() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let sub_dir = temp_dir.path().join("subdir");
    fs::create_dir_all(&sub_dir).expect("Failed to create dir");
    let file1 = temp_dir.path().join("file1.txt");
    let file2 = sub_dir.join("file2.txt");
    fs::write(&file1, "content1").expect("Failed to write file");
    fs::write(&file2, "content2").expect("Failed to write file");

    let paths = FileSystemUtils::walk_dir(temp_dir.path()).expect("Failed to walk dir");

    assert_eq!(paths.len(), 2);
}

#[test]
fn test_filehandle_open() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let file_path = temp_dir.path().join("test_file.txt");
    fs::write(&file_path, "test content").expect("Failed to write file");

    let handle = FileHandle::open(&file_path).expect("Failed to open file handle");

    assert_eq!(handle.path(), file_path);
}

#[test]
fn test_filehandle_read_write() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let file_path = temp_dir.path().join("test_file.txt");

    {
        let handle = FileHandle::create(&file_path).expect("Failed to create file handle");
        handle.write(b"Hello").expect("Failed to write");
    }

    {
        let handle = FileHandle::open(&file_path).expect("Failed to open file handle");
        let mut buf = [0u8; 5];
        let bytes_read = handle.read(&mut buf).expect("Failed to read");
        assert_eq!(bytes_read, 5);
        assert_eq!(&buf, b"Hello");
    }
}

#[test]
fn test_filehandle_seek() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let file_path = temp_dir.path().join("test_file.txt");
    fs::write(&file_path, "Hello, World!").expect("Failed to write file");

    let handle = FileHandle::open(&file_path).expect("Failed to open file handle");
    handle.seek(std::io::SeekFrom::Start(7)).expect("Failed to seek");

    let mut buf = [0u8; 6];
    let bytes_read = handle.read(&mut buf).expect("Failed to read");
    assert_eq!(bytes_read, 6);
    assert_eq!(&buf, b"World!");
}

#[test]
fn test_filehandle_attributes() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let file_path = temp_dir.path().join("test_file.txt");
    fs::write(&file_path, "test content").expect("Failed to write file");

    let handle = FileHandle::open(&file_path).expect("Failed to open file handle");
    let attrs = handle.attributes().expect("Failed to get attributes");

    assert_eq!(attrs.size, 12);
    assert!(attrs.is_file);
}

#[test]
fn test_filelock_exclusive() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let file_path = temp_dir.path().join("test_lock.txt");

    let _lock = FileLock::acquire_exclusive(&file_path).expect("Failed to acquire lock");
    assert!(FileSystemUtils::exists(&file_path));
}

#[test]
fn test_filelock_shared() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let file_path = temp_dir.path().join("test_lock.txt");

    let _lock = FileLock::acquire_shared(&file_path).expect("Failed to acquire lock");
    assert!(FileSystemUtils::exists(&file_path));
}

#[test]
fn test_filelock_path() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let file_path = temp_dir.path().join("test_lock.txt");

    let lock = FileLock::acquire_exclusive(&file_path).expect("Failed to acquire lock");
    assert_eq!(lock.path(), file_path);
}

#[test]
#[ignore]
fn test_filelock_is_exclusive() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let file_path = temp_dir.path().join("test_lock.txt");

    let exclusive_lock = FileLock::acquire_exclusive(&file_path).expect("Failed to acquire lock");
    assert!(exclusive_lock.is_exclusive());
}

#[test]
fn test_filecache_new() {
    let cache = FileCache::new(1024);
    assert_eq!(cache.size(), 0);
}

#[test]
fn test_filecache_put_and_get() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let file_path = temp_dir.path().join("test_file.txt");
    fs::write(&file_path, "test content").expect("Failed to write file");

    let cache = FileCache::new(10240);
    cache.put(&file_path, b"cached content".to_vec());

    assert_eq!(cache.size(), 14);
}

#[test]
fn test_filecache_get_or_load() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let file_path = temp_dir.path().join("test_file.txt");
    fs::write(&file_path, "test content").expect("Failed to write file");

    let cache = FileCache::new(10240);
    let data = cache.get_or_load(&file_path).expect("Failed to get or load");

    assert_eq!(&data, b"test content");
}

#[test]
fn test_filecache_remove() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let file_path = temp_dir.path().join("test_file.txt");
    fs::write(&file_path, "test content").expect("Failed to write file");

    let cache = FileCache::new(10240);
    cache.put(&file_path, b"cached content".to_vec());
    assert_eq!(cache.size(), 14);

    cache.remove(&file_path);
    assert_eq!(cache.size(), 0);
}

#[test]
fn test_filecache_clear() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let file_path1 = temp_dir.path().join("test_file1.txt");
    let file_path2 = temp_dir.path().join("test_file2.txt");
    fs::write(&file_path1, "content1").expect("Failed to write file");
    fs::write(&file_path2, "content2").expect("Failed to write file");

    let cache = FileCache::new(10240);
    cache.put(&file_path1, b"cached content1".to_vec());
    cache.put(&file_path2, b"cached content2".to_vec());

    cache.clear();
    assert_eq!(cache.size(), 0);
}

#[test]
fn test_filesystemwatcher_new() {
    let watcher = FileSystemWatcher::new();
    let changes = watcher.check_for_changes().expect("Failed to check changes");
    assert!(changes.is_empty());
}

#[test]
fn test_filesystemwatcher_watch() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let file_path = temp_dir.path().join("test_file.txt");
    fs::write(&file_path, "test content").expect("Failed to write file");

    let watcher = FileSystemWatcher::new();
    watcher.watch(&file_path).expect("Failed to watch");
}

#[test]
fn test_filesystemwatcher_unwatch() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let file_path = temp_dir.path().join("test_file.txt");
    fs::write(&file_path, "test content").expect("Failed to write file");

    let watcher = FileSystemWatcher::new();
    watcher.watch(&file_path).expect("Failed to watch");
    watcher.unwatch(&file_path);
}

#[test]
#[ignore]
fn test_filesystemwatcher_check_for_changes() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let file_path = temp_dir.path().join("test_file.txt");
    fs::write(&file_path, "initial content").expect("Failed to write file");

    let watcher = FileSystemWatcher::new();
    watcher.watch(&file_path).expect("Failed to watch");

    fs::write(&file_path, "modified content").expect("Failed to write file");

    let changes = watcher.check_for_changes().expect("Failed to check changes");
    assert!(!changes.is_empty());
}

#[test]
fn test_filesystemutils_exists_empty_path() {
    let empty_path = PathBuf::new();
    assert!(!FileSystemUtils::exists(&empty_path));
}

#[test]
fn test_filesystemutils_get_attributes_nonexistent() {
    let nonexistent = PathBuf::from("/nonexistent/path/file.txt");
    let result = FileSystemUtils::get_attributes(&nonexistent);
    assert!(result.is_err());
}

#[test]
fn test_filesystemutils_read_nonexistent() {
    let nonexistent = PathBuf::from("/nonexistent/path/file.txt");
    let result = FileSystemUtils::read_to_string(&nonexistent);
    assert!(result.is_err());
}

#[test]
fn test_filesystemutils_write_to_nonexistent_dir() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let nonexistent = temp_dir.path().join("nonexistent").join("file.txt");
    let result = FileSystemUtils::write(&nonexistent, "content");
    assert!(result.is_err());
}

#[test]
fn test_filesystemutils_read_dir_nonexistent() {
    let nonexistent = PathBuf::from("/nonexistent/path");
    let result = FileSystemUtils::read_dir(&nonexistent);
    assert!(result.is_err());
}

#[test]
fn test_filesystemutils_walk_dir_nonexistent() {
    let nonexistent = PathBuf::from("/nonexistent/path");
    let result = FileSystemUtils::walk_dir(&nonexistent);
    assert!(result.is_err());
}

#[test]
fn test_filesystemutils_remove_file_nonexistent() {
    let nonexistent = PathBuf::from("/nonexistent/path/file.txt");
    let result = FileSystemUtils::remove_file(&nonexistent);
    assert!(result.is_err());
}

#[test]
fn test_filesystemutils_remove_dir_nonexistent() {
    let nonexistent = PathBuf::from("/nonexistent/path");
    let result = FileSystemUtils::remove_dir(&nonexistent);
    assert!(result.is_err());
}

#[test]
fn test_filesystemutils_copy_nonexistent() {
    let nonexistent = PathBuf::from("/nonexistent/path/file.txt");
    let dest = PathBuf::from("/dest/file.txt");
    let result = FileSystemUtils::copy(&nonexistent, &dest);
    assert!(result.is_err());
}

#[test]
fn test_filesystemutils_copy_to_nonexistent() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let source = temp_dir.path().join("source.txt");
    let dest = temp_dir.path().join("nonexistent").join("dest.txt");
    fs::write(&source, "content").expect("Failed to write file");

    let result = FileSystemUtils::copy(&source, &dest);
    assert!(result.is_err());
}

#[test]
fn test_filesystemutils_rename_nonexistent() {
    let nonexistent = PathBuf::from("/nonexistent/path/file.txt");
    let dest = PathBuf::from("/dest/file.txt");
    let result = FileSystemUtils::rename(&nonexistent, &dest);
    assert!(result.is_err());
}

#[test]
fn test_filesystemutils_append_to_nonexistent() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let nonexistent = temp_dir.path().join("nonexistent").join("file.txt");
    let result = FileSystemUtils::append(&nonexistent, "content");
    assert!(result.is_err());
}

#[test]
fn test_fileattributes_debug() {
    let attrs = FileAttributes {
        size: 1024,
        created: std::time::SystemTime::UNIX_EPOCH,
        modified: std::time::SystemTime::UNIX_EPOCH,
        accessed: std::time::SystemTime::UNIX_EPOCH,
        is_directory: false,
        is_file: true,
        is_symlink: false,
        permissions: 0o644,
    };
    let debug_str = format!("{:?}", attrs);
    assert!(debug_str.contains("FileAttributes"));
}
