# FileSystemWatcher 动态分发消除方案

## 当前问题分析

### 代码位置
- 文件: `src/common/fs.rs`
- 第443行: `callback: Option<Arc<dyn Fn(&Path, FileEvent) + Send + Sync>>`
- 第517-522行: `set_callback` 方法

### 问题清单
1. **动态分发**: 使用 `dyn Fn` trait object，导致运行时虚表查找开销
2. **设计缺陷**: `set_callback(&mut self)` 破坏了不可变引用语义
3. **复杂性**: 用户需要在创建后才能设置回调，不够优雅
4. **内存成本**: Arc 指针额外开销

### 违反规范
违反 `AGENTS.md` 中的规则：
> Minimise the use of dynamic dispatch forms such as `dyn`, always prioritising deterministic types.
> All instances of dynamic dispatch must be explicitly documented in the `dynamic.md` file.

## 改进方案

### 方案1: 泛型回调（推荐）

**优点**: 零成本抽象，编译时多态，类型安全
**缺点**: 每个回调类型需要实例化一个新的泛型类型

#### 实现步骤

##### Step 1: 定义事件类型
```rust
#[derive(Debug, Clone)]
pub enum WatcherEvent {
    Changed(PathBuf, FileEvent),
    Error(PathBuf, String),
}
```

##### Step 2: 修改 FileSystemWatcher 结构
```rust
pub struct FileSystemWatcher<F>
where
    F: Fn(WatcherEvent) + Send + Sync + 'static,
{
    watched_paths: Arc<Mutex<HashMap<PathBuf, SystemTime>>>,
    callback: Arc<F>,
}
```

##### Step 3: 实现方法
```rust
impl<F> FileSystemWatcher<F>
where
    F: Fn(WatcherEvent) + Send + Sync + 'static,
{
    /// 创建带回调的文件系统监视器
    pub fn new(callback: F) -> Self {
        Self {
            watched_paths: Arc::new(Mutex::new(HashMap::new())),
            callback: Arc::new(callback),
        }
    }

    /// 添加监视路径
    pub fn watch<P: AsRef<Path>>(&self, path: P) -> FsResult<()> {
        let path_buf = path.to_path_buf();
        let metadata = fs::metadata(&path_buf).map_err(|e| FsError::IoError(e))?;
        let modified_time = metadata.modified().map_err(|_| {
            FsError::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Could not get modification time",
            ))
        })?;

        self.watched_paths
            .lock()
            .expect("File system watcher paths lock should not be poisoned")
            .insert(path_buf, modified_time);
        Ok(())
    }

    /// 检查变化（无需返回列表，直接调用回调）
    pub fn check_for_changes(&self) -> FsResult<()> {
        let watched_paths = self
            .watched_paths
            .lock()
            .expect("File system watcher paths lock should not be poisoned");

        for (path, last_modified) in watched_paths.iter() {
            if let Ok(metadata) = fs::metadata(path) {
                if let Ok(current_modified) = metadata.modified() {
                    if &current_modified != last_modified {
                        (self.callback)(WatcherEvent::Changed(
                            path.clone(),
                            FileEvent::Modified,
                        ));
                    }
                }
            } else {
                (self.callback)(WatcherEvent::Error(
                    path.clone(),
                    "File was deleted".to_string(),
                ));
            }
        }
        Ok(())
    }

    /// 移除监视路径
    pub fn unwatch<P: AsRef<Path>>(&self, path: P) {
        let path_buf = path.to_path_buf();
        self.watched_paths
            .lock()
            .expect("File system watcher paths lock should not be poisoned")
            .remove(&path_buf);
    }
}
```

#### 使用示例
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watcher_with_closure() {
        let watcher = FileSystemWatcher::new(|event| {
            match event {
                WatcherEvent::Changed(path, event) => {
                    println!("File changed: {:?} - {:?}", path, event);
                }
                WatcherEvent::Error(path, msg) => {
                    eprintln!("Error watching {}: {}", path.display(), msg);
                }
            }
        });

        // 使用 watcher...
    }

    #[test]
    fn test_watcher_with_custom_handler() {
        struct MyHandler {
            count: Arc<Mutex<u32>>,
        }

        impl MyHandler {
            fn handle_event(&self, event: WatcherEvent) {
                match event {
                    WatcherEvent::Changed(_, _) => {
                        let mut c = self.count.lock().unwrap();
                        *c += 1;
                    }
                    _ => {}
                }
            }
        }

        let handler = MyHandler {
            count: Arc::new(Mutex::new(0)),
        };
        let count_clone = handler.count.clone();
        
        let watcher = FileSystemWatcher::new(move |event| {
            handler.handle_event(event);
        });

        // 使用 watcher...
    }
}
```

### 方案2: 无回调版本（简化）

如果实时回调需求不强烈，可以完全移除回调机制，只返回变化列表。

#### 实现步骤

```rust
pub struct FileSystemWatcher {
    watched_paths: Arc<Mutex<HashMap<PathBuf, SystemTime>>>,
}

impl FileSystemWatcher {
    pub fn new() -> Self {
        Self {
            watched_paths: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn watch<P: AsRef<Path>>(&self, path: P) -> FsResult<()> {
        // 同上...
    }

    pub fn check_for_changes(&self) -> FsResult<Vec<(PathBuf, FileEvent)>> {
        let watched_paths = self
            .watched_paths
            .lock()
            .expect("File system watcher paths lock should not be poisoned");
        let mut changes = Vec::new();

        for (path, last_modified) in watched_paths.iter() {
            if let Ok(metadata) = fs::metadata(path) {
                if let Ok(current_modified) = metadata.modified() {
                    if &current_modified != last_modified {
                        changes.push((path.clone(), FileEvent::Modified));
                    }
                }
            } else {
                changes.push((path.clone(), FileEvent::Deleted));
            }
        }

        Ok(changes)
    }

    pub fn unwatch<P: AsRef<Path>>(&self, path: P) {
        // 同上...
    }
}
```

#### 使用示例
```rust
let watcher = FileSystemWatcher::new();
watcher.watch("/path/to/file")?;

loop {
    let changes = watcher.check_for_changes()?;
    for (path, event) in changes {
        println!("Changed: {:?} - {:?}", path, event);
    }
}
```

### 方案3: 枚举分发（折中方案）

如果需要支持多种回调实现但又想避免 `dyn`，可以使用枚举。

```rust
pub enum WatcherCallback {
    Log,
    Custom(Arc<dyn Fn(WatcherEvent) + Send + Sync>),
}

pub struct FileSystemWatcher {
    watched_paths: Arc<Mutex<HashMap<PathBuf, SystemTime>>>,
    callback: Option<WatcherCallback>,
}
```

**注意**: 这仍然包含 `dyn`，需要在 `docs/archive/dynamic.md` 中记录。

## 对比分析

| 方案 | 性能 | 灵活性 | 复杂性 | 推荐度 |
|------|------|--------|--------|--------|
| 方案1: 泛型回调 | ⭐⭐⭐⭐⭐ 零成本 | ⭐⭐⭐⭐ | ⭐⭐⭐ 中等 | ⭐⭐⭐⭐⭐ |
| 方案2: 无回调 | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐ 简单 | ⭐⭐⭐⭐ |
| 方案3: 枚举分发 | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ |

## 迁移检查清单

当集成 fs 模块时，需要检查：

- [ ] 是否有任何代码调用 `FileSystemWatcher::set_callback()`
- [ ] 是否需要支持动态回调（多个不同类型的回调）
- [ ] 是否需要在运行时切换回调实现
- [ ] 性能需求是否允许虚表查找开销
- [ ] 是否可以接受编译时多态的泛型代码膨胀

## 推荐方案

**首选: 方案1（泛型回调）**
- 完全符合 AGENTS.md 的零动态分发要求
- 编译时类型安全，运行时零开销
- 设计更优雅（在创建时传入回调）
- 支持任意 Fn 实现（闭包、函数指针、自定义类型）

**备选: 方案2（无回调）**
- 如果 `FileSystemWatcher` 主要用于在查询点获取变化
- 简化设计，易于测试和维护

## 文档要求

若选择**方案1**或**方案3**，需在 `docs/archive/dynamic.md` 中补充记录。

---

**创建时间**: 2025-12-23  
**分析来源**: src/common/fs.rs Line 443  
**待实现状态**: ⏳ 等待集成 fs 模块时决策
