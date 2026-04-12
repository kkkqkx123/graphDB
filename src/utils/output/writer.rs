//! Writer implementations for output module

use std::io::{self, BufWriter, Write};
use std::sync::Mutex;

/// A writer that outputs to stdout
pub struct StdoutWriter {
    stdout: io::Stdout,
}

impl StdoutWriter {
    /// Create a new stdout writer
    pub fn new() -> Self {
        Self { stdout: io::stdout() }
    }
}

impl Default for StdoutWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl Write for StdoutWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.stdout.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.stdout.flush()
    }
}

/// A writer that outputs to stderr
pub struct StderrWriter {
    stderr: io::Stderr,
}

impl StderrWriter {
    /// Create a new stderr writer
    pub fn new() -> Self {
        Self { stderr: io::stderr() }
    }
}

impl Default for StderrWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl Write for StderrWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.stderr.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.stderr.flush()
    }
}

/// A writer that outputs to a file with buffering
pub struct FileWriter {
    writer: BufWriter<std::fs::File>,
}

impl FileWriter {
    /// Create a new file writer
    pub fn new(file: std::fs::File) -> Self {
        Self {
            writer: BufWriter::new(file),
        }
    }

    /// Create a new file writer from path
    pub fn from_path(path: &std::path::Path, append: bool) -> io::Result<Self> {
        use std::fs::OpenOptions;

        let file = if append {
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(path)?
        } else {
            OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(path)?
        };

        Ok(Self::new(file))
    }
}

impl Write for FileWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.writer.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

/// A writer that outputs to multiple writers
pub struct MultiWriter {
    writers: Mutex<Vec<Box<dyn Write + Send>>>,
}

impl MultiWriter {
    /// Create a new multi-writer
    pub fn new() -> Self {
        Self {
            writers: Mutex::new(Vec::new()),
        }
    }

    /// Add a writer to the multi-writer
    pub fn add_writer<W: Write + Send + 'static>(&self, writer: W) {
        let mut writers = self.writers.lock().expect("lock poisoned");
        writers.push(Box::new(writer));
    }

    /// Create a multi-writer with initial writers
    pub fn with_writers<W: Write + Send + 'static>(writers: Vec<W>) -> Self {
        let multi = Self::new();
        for w in writers {
            multi.add_writer(w);
        }
        multi
    }
}

impl Default for MultiWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl Write for MultiWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut writers = self.writers.lock().expect("lock poisoned");
        for writer in writers.iter_mut() {
            writer.write_all(buf)?;
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        let mut writers = self.writers.lock().expect("lock poisoned");
        for writer in writers.iter_mut() {
            writer.flush()?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_multi_writer() {
        let cursor1 = Cursor::new(Vec::new());
        let cursor2 = Cursor::new(Vec::new());

        let mut multi = MultiWriter::with_writers(vec![cursor1, cursor2]);
        multi.write_all(b"hello").unwrap();
        multi.flush().unwrap();

        // Note: We can't easily verify the content without interior mutability
        // This test mainly checks that it compiles and runs without panic
    }
}
