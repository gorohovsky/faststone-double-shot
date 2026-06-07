//! FastStone Double Shot — delete paired JPG+RAW files together.
//!
//! The two binaries (`FsDelBin`, `FsDelPerm`) are thin shims over [`run_cli`];
//! all behaviour lives here so it can be unit-tested without touching the
//! Recycle Bin or popping real dialogs. The side effects — confirmation,
//! error reporting, deletion — are abstracted behind the [`Ui`] and [`Deleter`]
//! traits and injected into [`run`].

pub mod matching;
pub mod message;
pub mod mode;

#[cfg(windows)]
mod windows_os;

#[cfg(test)]
mod test_support;

use std::path::{Path, PathBuf};

pub use mode::Mode;

/// Process exit code when the deletion completes (or there was nothing to do).
const EXIT_OK: i32 = 0;
/// Process exit code when no existing file path was supplied.
const EXIT_NO_PATH: i32 = 1;

/// User interaction: confirming a deletion and reporting failures.
///
/// Abstracted so [`run`] can be tested with a fake instead of real dialogs.
pub trait Ui {
    /// Returns `true` when the user approves the deletion.
    fn confirm(&self, message: &str) -> bool;

    /// Reports files that could not be deleted.
    fn report_errors(&self, message: &str);
}

/// Deletes a single file according to the active [`Mode`].
///
/// Abstracted so [`run`] can be tested without destroying real files.
pub trait Deleter {
    fn delete(&self, path: &Path, mode: Mode) -> Result<(), String>;
}

/// The result of a deletion run — one variant per branch the user can hit.
#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    /// All matched files were deleted; carries the count.
    Deleted(usize),
    /// No supported file shared the selected file's base name.
    NothingToDelete,
    /// Confirmation was required and the user declined.
    Cancelled,
    /// The supplied path did not point at an existing file.
    PathNotFound,
    /// Some files were deleted; others failed (named in `failed`).
    PartialFailure { deleted: usize, failed: Vec<String> }
}

/// Finds every file paired with `input` and deletes them, confirming and
/// reporting errors through the injected [`Ui`].
pub fn run<U: Ui, D: Deleter>(input: &Path, mode: Mode, ui: &U, deleter: &D) -> Outcome {
    let Some(target) = resolve_existing_file(input) else {
        return Outcome::PathNotFound;
    };

    let matches = matching::find_matching_files(&target);
    if matches.is_empty() {
        return Outcome::NothingToDelete;
    }

    if mode.requires_confirmation(matches.len()) && !ui.confirm(&message::confirmation(&matches)) {
        return Outcome::Cancelled;
    }

    let outcome = delete_all(&matches, mode, deleter);
    if let Outcome::PartialFailure { failed, .. } = &outcome {
        ui.report_errors(&message::error(failed));
    }

    outcome
}

/// Entry point for the binaries: reads the file path from the command line,
/// wires up the real Windows side effects, and maps the [`Outcome`] to an exit
/// code.
#[cfg(windows)]
pub fn run_cli(mode: Mode) -> i32 {
    let Some(input) = std::env::args_os().nth(1) else {
        return EXIT_NO_PATH;
    };

    match run(Path::new(&input), mode, &windows_os::WindowsUi, &windows_os::WindowsDeleter) {
        Outcome::PathNotFound => EXIT_NO_PATH,
        _ => EXIT_OK
    }
}

fn delete_all<D: Deleter>(paths: &[PathBuf], mode: Mode, deleter: &D) -> Outcome {
    let mut deleted = 0;
    let mut failed = Vec::new();

    for path in paths {
        match deleter.delete(path, mode) {
            Ok(()) => deleted += 1,
            Err(_) => failed.push(file_name(path))
        }
    }

    if failed.is_empty() {
        return Outcome::Deleted(deleted);
    }

    Outcome::PartialFailure { deleted, failed }
}

fn resolve_existing_file(input: &Path) -> Option<PathBuf> {
    let absolute = std::path::absolute(input).ok()?;

    absolute.is_file().then_some(absolute)
}

fn file_name(path: &Path) -> String {
    path.file_name().unwrap_or_default().to_string_lossy().into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{touch, TempDir};
    use std::cell::RefCell;

    struct FakeUi {
        approve: bool,
        confirmed: RefCell<bool>,
        errors: RefCell<Vec<String>>
    }

    impl FakeUi {
        fn approving() -> Self {
            Self::new(true)
        }

        fn declining() -> Self {
            Self::new(false)
        }

        fn new(approve: bool) -> Self {
            Self { approve, confirmed: RefCell::new(false), errors: RefCell::new(Vec::new()) }
        }
    }

    impl Ui for FakeUi {
        fn confirm(&self, _message: &str) -> bool {
            *self.confirmed.borrow_mut() = true;
            self.approve
        }

        fn report_errors(&self, message: &str) {
            self.errors.borrow_mut().push(message.to_string());
        }
    }

    struct FakeDeleter {
        fail: Vec<String>,
        deleted: RefCell<Vec<String>>
    }

    impl FakeDeleter {
        fn new() -> Self {
            Self { fail: Vec::new(), deleted: RefCell::new(Vec::new()) }
        }

        fn failing(names: &[&str]) -> Self {
            Self { fail: names.iter().map(|name| name.to_string()).collect(), deleted: RefCell::new(Vec::new()) }
        }
    }

    impl Deleter for FakeDeleter {
        fn delete(&self, path: &Path, _mode: Mode) -> Result<(), String> {
            let name = file_name(path);
            if self.fail.contains(&name) {
                return Err("boom".to_string());
            }

            self.deleted.borrow_mut().push(name);
            Ok(())
        }
    }

    #[test]
    fn deletes_a_pair_silently_in_recycle_mode() {
        let directory = TempDir::new();
        let jpg = touch(&directory, "shot.jpg");
        touch(&directory, "shot.cr2");
        let ui = FakeUi::declining();
        let deleter = FakeDeleter::new();

        let outcome = run(&jpg, Mode::RecycleBin, &ui, &deleter);

        assert_eq!(outcome, Outcome::Deleted(2));
        assert!(!*ui.confirmed.borrow());
        assert_eq!(*deleter.deleted.borrow(), ["shot.cr2", "shot.jpg"]);
    }

    #[test]
    fn confirms_before_deleting_three_or_more_in_recycle_mode() {
        let directory = TempDir::new();
        let jpg = touch(&directory, "burst.jpg");
        touch(&directory, "burst.cr2");
        touch(&directory, "burst.nef");
        let ui = FakeUi::approving();

        let outcome = run(&jpg, Mode::RecycleBin, &ui, &FakeDeleter::new());

        assert_eq!(outcome, Outcome::Deleted(3));
        assert!(*ui.confirmed.borrow());
    }

    #[test]
    fn permanent_mode_confirms_even_for_a_single_file() {
        let directory = TempDir::new();
        let jpg = touch(&directory, "shot.jpg");
        let ui = FakeUi::approving();

        let outcome = run(&jpg, Mode::Permanent, &ui, &FakeDeleter::new());

        assert_eq!(outcome, Outcome::Deleted(1));
        assert!(*ui.confirmed.borrow());
    }

    #[test]
    fn declining_the_confirmation_deletes_nothing() {
        let directory = TempDir::new();
        let jpg = touch(&directory, "shot.jpg");
        let deleter = FakeDeleter::new();

        let outcome = run(&jpg, Mode::Permanent, &FakeUi::declining(), &deleter);

        assert_eq!(outcome, Outcome::Cancelled);
        assert!(deleter.deleted.borrow().is_empty());
    }

    #[test]
    fn reports_a_missing_path() {
        let directory = TempDir::new();
        let missing = directory.path().join("ghost.jpg");

        let outcome = run(&missing, Mode::RecycleBin, &FakeUi::approving(), &FakeDeleter::new());

        assert_eq!(outcome, Outcome::PathNotFound);
    }

    #[test]
    fn reports_nothing_to_delete_for_an_unsupported_file() {
        let directory = TempDir::new();
        let tiff = touch(&directory, "scan.tiff");

        let outcome = run(&tiff, Mode::RecycleBin, &FakeUi::approving(), &FakeDeleter::new());

        assert_eq!(outcome, Outcome::NothingToDelete);
    }

    #[test]
    fn reports_files_that_could_not_be_deleted() {
        let directory = TempDir::new();
        let jpg = touch(&directory, "shot.jpg");
        touch(&directory, "shot.cr2");
        let ui = FakeUi::approving();
        let deleter = FakeDeleter::failing(&["shot.cr2"]);

        let outcome = run(&jpg, Mode::RecycleBin, &ui, &deleter);

        assert_eq!(outcome, Outcome::PartialFailure { deleted: 1, failed: vec!["shot.cr2".to_string()] });
        assert_eq!(*deleter.deleted.borrow(), ["shot.jpg"]);
        assert_eq!(ui.errors.borrow().len(), 1);
    }
}
