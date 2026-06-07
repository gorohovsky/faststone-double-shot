//! Windows implementations of the [`Ui`] and [`Deleter`] side effects.
//!
//! This is the boundary where the program touches the operating system:
//! confirmation/error dialogs via `MessageBoxW` (user32), the Recycle Bin via
//! `SHFileOperationW` (shell32), and permanent deletes via [`std::fs`]. The two
//! functions are declared directly so the crate needs no external dependencies
//! and links against the import libraries shipped with the toolchain. This
//! layer is kept deliberately thin and is exercised by the manual checks in
//! TESTING.md rather than unit tests.

use std::ffi::{c_void, OsStr};
use std::os::windows::ffi::OsStrExt;
use std::path::Path;

use crate::{Deleter, Mode, Ui};

// MessageBoxW styles and result code (winuser.h).
const MB_OK: u32 = 0x0000_0000;
const MB_YESNO: u32 = 0x0000_0004;
const MB_ICONERROR: u32 = 0x0000_0010;
const MB_ICONWARNING: u32 = 0x0000_0030;
const MB_SETFOREGROUND: u32 = 0x0001_0000;
const IDYES: i32 = 6;

// SHFileOperationW function code and flags (shellapi.h).
const FO_DELETE: u32 = 0x0003;
const FOF_SILENT: u16 = 0x0004;
const FOF_NOCONFIRMATION: u16 = 0x0010;
const FOF_ALLOWUNDO: u16 = 0x0040;
const FOF_NOERRORUI: u16 = 0x0400;

const CONFIRM_TITLE: &str = "Confirm Delete";
const ERROR_TITLE: &str = "Delete Error";

/// Mirror of the Win32 `SHFILEOPSTRUCTW` (shellapi.h). `#[repr(C)]` reproduces
/// the exact field layout the API expects.
#[repr(C)]
struct ShFileOpStructW {
    window: *mut c_void,
    func: u32,
    from: *const u16,
    to: *const u16,
    flags: u16,
    any_operations_aborted: i32,
    name_mappings: *mut c_void,
    progress_title: *const u16
}

#[link(name = "user32")]
extern "system" {
    fn MessageBoxW(window: *mut c_void, text: *const u16, caption: *const u16, style: u32) -> i32;
}

#[link(name = "shell32")]
extern "system" {
    fn SHFileOperationW(operation: *mut ShFileOpStructW) -> i32;
}

/// Confirmation and error dialogs backed by the Win32 `MessageBoxW`.
pub struct WindowsUi;

impl Ui for WindowsUi {
    fn confirm(&self, message: &str) -> bool {
        message_box(message, CONFIRM_TITLE, MB_YESNO | MB_ICONWARNING | MB_SETFOREGROUND) == IDYES
    }

    fn report_errors(&self, message: &str) {
        message_box(message, ERROR_TITLE, MB_OK | MB_ICONERROR | MB_SETFOREGROUND);
    }
}

/// Deletes one file to the Recycle Bin or permanently, per [`Mode`].
pub struct WindowsDeleter;

impl Deleter for WindowsDeleter {
    fn delete(&self, path: &Path, mode: Mode) -> Result<(), String> {
        match mode {
            Mode::RecycleBin => recycle(path),
            Mode::Permanent => std::fs::remove_file(path).map_err(|error| error.to_string())
        }
    }
}

fn message_box(text: &str, title: &str, style: u32) -> i32 {
    let text = to_wide(text);
    let title = to_wide(title);

    // SAFETY: both buffers are null-terminated and live for the whole call.
    unsafe { MessageBoxW(std::ptr::null_mut(), text.as_ptr(), title.as_ptr(), style) }
}

fn recycle(path: &Path) -> Result<(), String> {
    let source = double_null_terminated(path);

    // SAFETY: a zeroed struct is valid here — every pointer field is null until
    // set below, and `from` then points at the double-null-terminated buffer.
    let mut operation: ShFileOpStructW = unsafe { std::mem::zeroed() };
    operation.func = FO_DELETE;
    operation.from = source.as_ptr();
    operation.flags = FOF_ALLOWUNDO | FOF_NOCONFIRMATION | FOF_NOERRORUI | FOF_SILENT;

    // SAFETY: `operation` is fully initialised and `source` outlives the call.
    let result = unsafe { SHFileOperationW(&mut operation) };
    if result == 0 && operation.any_operations_aborted == 0 {
        return Ok(());
    }

    Err(format!("Recycle Bin operation failed (code {result})"))
}

fn to_wide(text: &str) -> Vec<u16> {
    OsStr::new(text).encode_wide().chain(std::iter::once(0)).collect()
}

fn double_null_terminated(path: &Path) -> Vec<u16> {
    path.as_os_str().encode_wide().chain([0, 0]).collect()
}
