use std::{io::Write, sync::Mutex};
use tauri::{Emitter, Manager};

struct ProbeState(Mutex<Vec<String>>);

fn record(state: &ProbeState, message: String) {
    let line = format!("{:?} {message}", std::time::SystemTime::now());
    if let Ok(mut rows) = state.0.lock() { rows.push(line.clone()); }
    if let Ok(path) = std::env::var("LITTLETODO_PROBE_LOG") {
        if let Ok(mut file) = std::fs::OpenOptions::new().append(true).create(true).open(path) {
            let _ = writeln!(file, "{line}");
        }
    }
}

#[cfg(target_os = "windows")]
mod desktop {
    use std::ffi::c_void;
    type Hwnd = *mut c_void;
    #[link(name = "user32")]
    unsafe extern "system" {
        fn EnumWindows(callback: unsafe extern "system" fn(Hwnd, isize) -> i32, param: isize) -> i32;
        fn FindWindowExW(parent: Hwnd, after: Hwnd, class: *const u16, title: *const u16) -> Hwnd;
        fn GetWindowLongPtrW(hwnd: Hwnd, index: i32) -> isize;
        fn SetWindowLongPtrW(hwnd: Hwnd, index: i32, value: isize) -> isize;
        fn SetParent(hwnd: Hwnd, parent: Hwnd) -> Hwnd;
        fn GetParent(hwnd: Hwnd) -> Hwnd;
        fn SetWindowPos(hwnd: Hwnd, after: Hwnd, x: i32, y: i32, w: i32, h: i32, flags: u32) -> i32;
    }
    unsafe extern "system" fn find_host(hwnd: Hwnd, result: isize) -> i32 {
        let class: Vec<u16> = "SHELLDLL_DefView\0".encode_utf16().collect();
        if !unsafe { FindWindowExW(hwnd, std::ptr::null_mut(), class.as_ptr(), std::ptr::null()) }.is_null() {
            unsafe { *(result as *mut Hwnd) = hwnd; }
            return 0;
        }
        1
    }
    // Matches the successful native probe: reuse the existing icon host, no shell messages or hooks.
    pub fn attach(raw: isize) -> Result<String, String> {
        let hwnd = raw as Hwnd;
        let mut host: Hwnd = std::ptr::null_mut();
        unsafe {
            EnumWindows(find_host, (&mut host as *mut Hwnd) as isize);
            if host.is_null() { return Err("No existing desktop icon host".into()); }
            let old = GetWindowLongPtrW(hwnd, -16);
            SetWindowLongPtrW(hwnd, -16, (old & !(0x8000_0000_u32 as isize)) | 0x4000_0000);
            SetParent(hwnd, host);
            if GetParent(hwnd) != host {
                SetWindowLongPtrW(hwnd, -16, old);
                return Err("SetParent verification failed".into());
            }
            if SetWindowPos(hwnd, std::ptr::null_mut(), 1500, 80, 360, 310, 0x0040 | 0x0020 | 0x0010) == 0 {
                return Err("Desktop attached but positioning failed".into());
            }
        }
        Ok(format!("attached=true hwnd={raw} parent={}", host as isize))
    }
    pub fn move_panel(raw: isize, x: i32, y: i32) -> Result<(), String> {
        let ok = unsafe { SetWindowPos(raw as Hwnd, std::ptr::null_mut(), x, y, 0, 0, 0x0001 | 0x0004 | 0x0010) };
        if ok == 0 { Err("Move failed".into()) } else { Ok(()) }
    }
}

#[tauri::command]
fn attach(app: tauri::AppHandle, window: tauri::WebviewWindow, state: tauri::State<ProbeState>) -> Result<String, String> {
    if window.label() != "main" { return Err("Controller only".into()); }
    let panel = app.get_webview_window("panel").ok_or("Panel not found")?;
    #[cfg(target_os = "windows")]
    let result = desktop::attach(panel.hwnd().map_err(|e| e.to_string())?.0 as isize);
    #[cfg(not(target_os = "windows"))]
    let result: Result<String, String> = Err("Windows-only desktop adapter; macOS unverified".into());
    if result.is_ok() { let _ = panel.set_skip_taskbar(true); }
    record(&state, format!("attach={result:?}"));
    result
}

#[tauri::command]
fn appearance(app: tauri::AppHandle, window: tauri::WebviewWindow, alpha: f64, blue: bool, state: tauri::State<ProbeState>) -> Result<(), String> {
    if window.label() != "main" || !alpha.is_finite() || !(0.4..=1.0).contains(&alpha) { return Err("Invalid appearance request".into()); }
    app.emit_to("panel", "appearance", serde_json::json!({"alpha":alpha,"blue":blue})).map_err(|e| e.to_string())?;
    record(&state, format!("appearance_requested alpha={alpha} blue={blue}"));
    Ok(())
}

#[tauri::command]
fn move_panel(window: tauri::WebviewWindow, x: i32, y: i32) -> Result<(), String> {
    if window.label() != "panel" || !(-10000..=20000).contains(&x) || !(-10000..=20000).contains(&y) { return Err("Invalid move".into()); }
    #[cfg(target_os = "windows")]
    { desktop::move_panel(window.hwnd().map_err(|e| e.to_string())?.0 as isize, x, y) }
    #[cfg(not(target_os = "windows"))]
    { Err("Windows-only probe".into()) }
}

#[tauri::command]
async fn open_detail(app: tauri::AppHandle, state: tauri::State<'_, ProbeState>) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("detail") { window.show().map_err(|e| e.to_string())?; window.set_focus().map_err(|e| e.to_string())?; }
    else {
        tauri::WebviewWindowBuilder::new(&app, "detail", tauri::WebviewUrl::App("index.html".into()))
            .title("LittleTodo Tauri Detail Probe").inner_size(480.0,360.0).build().map_err(|e| e.to_string())?;
    }
    record(&state, "detail_opened".into());
    Ok(())
}

#[tauri::command]
fn event(window: tauri::WebviewWindow, name: String, state: tauri::State<ProbeState>) -> Result<(), String> {
    if !["ready", "checkbox_on", "checkbox_off", "drag_end", "detail_edited", "appearance_applied"].contains(&name.as_str()) { return Err("Unknown probe event".into()); }
    record(&state, format!("{}:{name}", window.label()));
    Ok(())
}

#[tauri::command]
fn status(state: tauri::State<ProbeState>) -> Vec<String> { state.0.lock().map(|s| s.clone()).unwrap_or_default() }

#[tauri::command]
fn quit(app: tauri::AppHandle, window: tauri::WebviewWindow) { if window.label()=="main" { app.exit(0); } }

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(ProbeState(Mutex::new(Vec::new())))
        .invoke_handler(tauri::generate_handler![attach, appearance, move_panel, open_detail, event, status, quit])
        .run(tauri::generate_context!())
        .expect("probe runtime failed");
}
