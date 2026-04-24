use crate::config::{self, Config};
use crate::error::{AppError, AppResult};
use crate::launcher;
use crate::scanner::{self, WorkspaceEntry};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::State;

pub struct AppState {
    pub config: Mutex<Config>,
    pub config_path: PathBuf,
}

impl AppState {
    pub fn new(config_path: PathBuf) -> AppResult<Self> {
        let config = config::load_from(&config_path)?;
        Ok(Self {
            config: Mutex::new(config),
            config_path,
        })
    }

    fn persist(&self) -> AppResult<()> {
        let cfg = self.config.lock().unwrap().clone();
        config::save_to(&self.config_path, &cfg)?;
        Ok(())
    }
}

#[tauri::command]
pub fn get_config(state: State<'_, AppState>) -> Config {
    state.config.lock().unwrap().clone()
}

#[tauri::command]
pub fn set_root_folder(state: State<'_, AppState>, path: Option<PathBuf>) -> AppResult<Config> {
    {
        let mut cfg = state.config.lock().unwrap();
        cfg.root_folder = path;
    }
    state.persist()?;
    Ok(state.config.lock().unwrap().clone())
}

#[tauri::command]
pub fn get_workspaces(state: State<'_, AppState>) -> AppResult<Vec<WorkspaceEntry>> {
    let root = state.config.lock().unwrap().root_folder.clone();
    match root {
        Some(p) => Ok(scanner::scan(&p)?),
        None => Ok(Vec::new()),
    }
}

#[tauri::command]
pub fn launch(path: PathBuf) -> AppResult<()> {
    if !path.exists() {
        return Err(AppError::Message(format!(
            "Workspace file no longer exists: {}",
            path.display()
        )));
    }
    launcher::open(&path)?;
    Ok(())
}

#[tauri::command]
pub fn set_pinned(state: State<'_, AppState>, path: PathBuf, pinned: bool) -> AppResult<Config> {
    {
        let mut cfg = state.config.lock().unwrap();
        cfg.pinned.retain(|p| p != &path);
        if pinned {
            cfg.pinned.push(path);
        }
    }
    state.persist()?;
    Ok(state.config.lock().unwrap().clone())
}

#[tauri::command]
pub fn set_icon(
    state: State<'_, AppState>,
    workspace: PathBuf,
    icon: Option<PathBuf>,
) -> AppResult<Config> {
    {
        let mut cfg = state.config.lock().unwrap();
        match icon {
            Some(i) => {
                cfg.icons.insert(workspace, i);
            }
            None => {
                cfg.icons.remove(&workspace);
            }
        }
    }
    state.persist()?;
    Ok(state.config.lock().unwrap().clone())
}

#[tauri::command]
pub fn resolved_code_binary() -> Option<PathBuf> {
    launcher::resolve_code_binary()
}

#[tauri::command]
pub fn focus_window(hwnd: i64) -> AppResult<()> {
    #[cfg(windows)]
    unsafe {
        win::force_foreground(hwnd);
    }
    let _ = hwnd; // silence unused on non-windows
    Ok(())
}

#[cfg(windows)]
mod win {
    #[link(name = "user32")]
    extern "system" {
        fn IsWindow(hwnd: *mut core::ffi::c_void) -> i32;
        fn IsIconic(hwnd: *mut core::ffi::c_void) -> i32;
        fn ShowWindow(hwnd: *mut core::ffi::c_void, n_cmd_show: i32) -> i32;
        fn SetForegroundWindow(hwnd: *mut core::ffi::c_void) -> i32;
        fn BringWindowToTop(hwnd: *mut core::ffi::c_void) -> i32;
        fn AllowSetForegroundWindow(dw_process_id: u32) -> i32;
    }
    const SW_RESTORE: i32 = 9;
    const ASFW_ANY: u32 = 0xFFFF_FFFF;

    pub unsafe fn force_foreground(hwnd: i64) {
        let hwnd = hwnd as isize as *mut core::ffi::c_void;
        if IsWindow(hwnd) == 0 {
            return;
        }
        // Grant any process permission to take foreground (belt + braces —
        // SetForegroundWindow from our own foreground process should work
        // anyway, but some Windows configurations are strict).
        AllowSetForegroundWindow(ASFW_ANY);
        // If the window is minimized, restore it first.
        if IsIconic(hwnd) != 0 {
            ShowWindow(hwnd, SW_RESTORE);
        }
        BringWindowToTop(hwnd);
        SetForegroundWindow(hwnd);
    }
}
