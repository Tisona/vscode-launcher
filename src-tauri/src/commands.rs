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
