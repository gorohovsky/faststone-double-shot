//! Shared unit-test helpers: a dependency-free temporary directory.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

/// A uniquely-named temporary directory, removed when dropped. Hand-rolled so
/// tests pull in no external crate (e.g. `tempfile`).
pub(crate) struct TempDir {
    path: PathBuf
}

impl TempDir {
    pub(crate) fn new() -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let sequence = COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("fds_test_{}_{}", std::process::id(), sequence));
        std::fs::create_dir_all(&path).unwrap();

        Self { path }
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

/// Creates an empty file named `name` inside `directory`, returning its path.
pub(crate) fn touch(directory: &TempDir, name: &str) -> PathBuf {
    let path = directory.path().join(name);
    std::fs::File::create(&path).unwrap();

    path
}
