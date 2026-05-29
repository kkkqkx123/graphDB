//! WORM (Write Once Read Many) directory abstraction.
//!
//! The `WriterKind` and `WritePtr` enums intentionally expose crate-internal
//! writer types (`SafeFileWriter`, `VecWriter`, `FooterProxy`) in their variants.
//! These inner types are `pub(crate)` because they are implementation details
//! of specific `Directory` backends. External users interact with them via
//! `Write` and `TerminatingWrite` trait methods.
// These inner types are intentionally `pub(crate)` — they are implementation
// details of specific Directory backends, exposed through enum variant fields.
#![allow(private_interfaces)]

#[cfg(feature = "mmap")]
mod mmap_directory;

mod directory;
mod directory_lock;
pub mod footer;
mod managed_directory;
mod ram_directory;
mod watch_event_router;

/// Errors specific to the directory module.
pub mod error;

mod composite_file;

use std::io::{self, BufWriter, Write};
use std::path::PathBuf;

pub use common::file_slice::{FileHandle, FileSlice};
pub use common::{AntiCallToken, OwnedBytes, TerminatingWrite};

pub use self::composite_file::{CompositeFile, CompositeWrite};
pub use self::directory::{Directory, DirectoryClone, DirectoryLock};
pub use self::directory_lock::{Lock, INDEX_WRITER_LOCK, META_LOCK};
pub use self::ram_directory::RamDirectory;
pub use self::watch_event_router::{WatchCallback, WatchCallbackList, WatchHandle};

/// Outcome of the Garbage collection
pub struct GarbageCollectionResult {
    /// List of files that were deleted in this cycle
    pub deleted_files: Vec<PathBuf>,
    /// List of files that were schedule to be deleted in this cycle,
    /// but deletion did not work. This typically happens on windows,
    /// as deleting a memory mapped file is forbidden.
    ///
    /// If a searcher is still held, a file cannot be deleted.
    /// This is not considered a bug, the file will simply be deleted
    /// in the next GC.
    pub failed_to_delete_files: Vec<PathBuf>,
}

#[cfg(all(feature = "mmap", unix))]
pub use memmap2::Advice;

pub use self::managed_directory::ManagedDirectory;
#[cfg(feature = "mmap")]
pub use self::mmap_directory::MmapDirectory;

/// Concrete writer implementations used by [`WritePtr`].
///
/// Enum-based dispatch replaces `Box<dyn TerminatingWrite>`, eliminating
/// vtable calls in the hot write path while preserving type safety.
pub enum WriterKind {
    /// File-backed writer (MmapDirectory).
    #[cfg(feature = "mmap")]
    File(mmap_directory::SafeFileWriter),
    /// In-memory Vec-backed writer (RamDirectory, testing).
    Vec(ram_directory::VecWriter),
}

impl Write for WriterKind {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            #[cfg(feature = "mmap")]
            WriterKind::File(w) => w.write(buf),
            WriterKind::Vec(w) => w.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            #[cfg(feature = "mmap")]
            WriterKind::File(w) => w.flush(),
            WriterKind::Vec(w) => w.flush(),
        }
    }
}

impl TerminatingWrite for WriterKind {
    fn terminate_ref(&mut self, token: AntiCallToken) -> io::Result<()> {
        match self {
            #[cfg(feature = "mmap")]
            WriterKind::File(w) => w.terminate_ref(token),
            WriterKind::Vec(w) => w.terminate_ref(token),
        }
    }
}

/// Write object for Directory.
///
/// Provides static dispatch for all writer configurations:
/// - `Plain`: used by MmapDirectory and RamDirectory (no CRC footer).
/// - `Managed`: used by ManagedDirectory (wraps in FooterProxy for CRC).
///
/// `WritePtr` is required to implement both Write and TerminatingWrite.
pub enum WritePtr {
    /// Unwrapped writer (MmapDirectory / RamDirectory).
    Plain(BufWriter<WriterKind>),
    /// CRC-wrapped writer (ManagedDirectory).
    Managed(BufWriter<footer::FooterProxy<WriterKind>>),
}

impl Write for WritePtr {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            WritePtr::Plain(w) => w.write(buf),
            WritePtr::Managed(w) => w.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            WritePtr::Plain(w) => w.flush(),
            WritePtr::Managed(w) => w.flush(),
        }
    }
}

impl TerminatingWrite for WritePtr {
    fn terminate_ref(&mut self, token: AntiCallToken) -> io::Result<()> {
        match self {
            WritePtr::Plain(w) => w.terminate_ref(token),
            WritePtr::Managed(w) => w.terminate_ref(token),
        }
    }
}

#[cfg(test)]
mod tests;
