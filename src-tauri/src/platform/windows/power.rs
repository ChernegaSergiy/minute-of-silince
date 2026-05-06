//! Windows-specific power management.
//! Handles system-wide power events like waking from sleep.

use std::sync::OnceLock;
use tauri::{AppHandle, Emitter, WebviewWindow};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    CallWindowProcW, SetWindowLongPtrW, GWLP_WNDPROC, WM_POWERBROADCAST,
};

static ORIGINAL_WNDPROC: OnceLock<isize> = OnceLock::new();
static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();

const PBT_APMRESUMEAUTOMATIC: usize = 0x0012;
const PBT_APMRESUMESUSPEND: usize = 0x0007;

/// Register a hook to listen for WM_POWERBROADCAST events.
pub fn register_power_hook(window: &WebviewWindow) {
    let hwnd = window.hwnd().expect("Failed to get HWND").0;
    let handle = window.app_handle().clone();
    let _ = APP_HANDLE.set(handle);

    unsafe {
        let original = SetWindowLongPtrW(
            HWND(hwnd as *mut _),
            GWLP_WNDPROC,
            wndproc as *const () as isize,
        );
        let _ = ORIGINAL_WNDPROC.set(original);
    }
}

/// Window procedure to handle WM_POWERBROADCAST.
unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if msg == WM_POWERBROADCAST {
        let wp = wparam.0;
        if wp == PBT_APMRESUMEAUTOMATIC || wp == PBT_APMRESUMESUSPEND {
            log::info!("System resume from sleep detected (WP: {wp})");
            if let Some(handle) = APP_HANDLE.get() {
                let _ = handle.emit("resume-from-sleep", ());
            }
        }
    }

    let original = ORIGINAL_WNDPROC.get().expect("Original WndProc not set");
    CallWindowProcW(
        Some(std::mem::transmute::<
            isize,
            unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT,
        >(*original)),
        hwnd,
        msg,
        wparam,
        lparam,
    )
}
