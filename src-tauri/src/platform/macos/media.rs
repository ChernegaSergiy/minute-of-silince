//! Pause / resume media players on macOS via Swift helper and NSWorkspace.

use crate::error::Result;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

unsafe extern "C" {
    fn macos_pause_all() -> *mut c_char;
    fn macos_resume_players(bundle_ids_csv: *const c_char);
    fn macos_free_string(ptr: *mut c_char);
}

pub async fn pause_all() -> Result<Vec<String>> {
    let ptr = unsafe { macos_pause_all() };
    if ptr.is_null() {
        return Ok(Vec::new());
    }

    let c_str = unsafe { CStr::from_ptr(ptr) };
    let str_slice = c_str.to_string_lossy();
    let bundle_ids: Vec<String> = str_slice
        .split(',')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();

    unsafe { macos_free_string(ptr) };
    Ok(bundle_ids)
}

pub async fn resume_specific(players: Vec<String>) -> Result<()> {
    if players.is_empty() {
        return Ok(());
    }

    let csv = players.join(",");
    if let Ok(c_string) = CString::new(csv) {
        unsafe { macos_resume_players(c_string.as_ptr()) };
    }
    Ok(())
}
