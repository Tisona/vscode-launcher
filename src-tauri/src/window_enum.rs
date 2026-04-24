//! Enumerate top-level OS windows and extract VSCode workspace display names.
//!
//! Why: VSCode's main Electron process argv only reflects the FIRST workspace it
//! was launched with. Subsequently-opened workspaces become new renderer windows
//! under the same main, and we cannot see them via argv alone. Enumerating
//! top-level windows and parsing their titles catches every open workspace.

/// Given a top-level window title, return the workspace display name if this
/// window appears to be a VSCode workspace window. Returns None for non-VSCode
/// windows, single-folder VSCode windows (no `.code-workspace` file), or titles
/// we cannot confidently parse.
#[cfg_attr(not(windows), allow(dead_code))]
pub fn extract_workspace_name_from_title(title: &str) -> Option<String> {
    // 1. Must end with " - Visual Studio Code".
    let without_vscode = title.strip_suffix(" - Visual Studio Code")?;
    // 2. Must end with " (Workspace)" (trim trailing whitespace first for
    //    flexibility on weird double-space variants).
    let trimmed = without_vscode.trim_end();
    let without_workspace = trimmed.strip_suffix("(Workspace)")?.trim_end();
    // 3. Take substring after the last " - " (separates active-editor from name).
    let name_raw = match without_workspace.rfind(" - ") {
        Some(idx) => &without_workspace[idx + 3..],
        None => without_workspace,
    };
    // 4. Trim leading non-alphanumeric chars (bullets, dots, whitespace).
    let name = name_raw
        .trim_start_matches(|c: char| !c.is_alphanumeric())
        .trim()
        .to_string();
    // 5. Empty → None.
    if name.is_empty() {
        return None;
    }
    Some(name)
}

#[derive(Debug, Clone)]
pub struct WorkspaceWindow {
    pub pid: u32,
    pub workspace_name: String,
}

#[cfg(windows)]
pub fn enumerate_workspace_windows() -> Vec<WorkspaceWindow> {
    let mut result: Vec<WorkspaceWindow> = Vec::new();
    let result_ptr: *mut Vec<WorkspaceWindow> = &mut result;
    unsafe {
        winapi::EnumWindows(enum_proc, result_ptr as isize);
    }
    result
}

#[cfg(not(windows))]
pub fn enumerate_workspace_windows() -> Vec<WorkspaceWindow> {
    Vec::new()
}

#[cfg(windows)]
unsafe extern "system" fn enum_proc(hwnd: winapi::HWND, lparam: isize) -> i32 {
    if winapi::IsWindowVisible(hwnd) == 0 {
        return 1;
    }
    let title_len = winapi::GetWindowTextLengthW(hwnd);
    if title_len <= 0 {
        return 1;
    }
    let mut buf: Vec<u16> = vec![0u16; title_len as usize + 1];
    let copied = winapi::GetWindowTextW(hwnd, buf.as_mut_ptr(), buf.len() as i32);
    if copied <= 0 {
        return 1;
    }
    let title = String::from_utf16_lossy(&buf[..copied as usize]);
    let Some(name) = extract_workspace_name_from_title(&title) else {
        return 1;
    };
    let mut pid: u32 = 0;
    winapi::GetWindowThreadProcessId(hwnd, &mut pid);
    let out = &mut *(lparam as *mut Vec<WorkspaceWindow>);
    out.push(WorkspaceWindow {
        pid,
        workspace_name: name,
    });
    1
}

#[cfg(windows)]
mod winapi {
    use std::ffi::c_int;

    pub type HWND = *mut core::ffi::c_void;
    pub type LPARAM = isize;
    pub type BOOL = c_int;
    pub type WNDENUMPROC = unsafe extern "system" fn(HWND, LPARAM) -> BOOL;

    #[link(name = "user32")]
    extern "system" {
        pub fn EnumWindows(lp_enum_func: WNDENUMPROC, l_param: LPARAM) -> BOOL;
        pub fn IsWindowVisible(hwnd: HWND) -> BOOL;
        pub fn GetWindowTextLengthW(hwnd: HWND) -> c_int;
        pub fn GetWindowTextW(hwnd: HWND, lp_string: *mut u16, n_max_count: c_int) -> c_int;
        pub fn GetWindowThreadProcessId(hwnd: HWND, lp_dw_process_id: *mut u32) -> u32;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_workspace_window() {
        assert_eq!(
            extract_workspace_name_from_title("personal (Workspace) - Visual Studio Code"),
            Some("personal".to_string())
        );
    }

    #[test]
    fn bullet_prefix_unsaved() {
        assert_eq!(
            extract_workspace_name_from_title("\u{25cf} personal (Workspace) - Visual Studio Code"),
            Some("personal".to_string())
        );
    }

    #[test]
    fn active_editor_prefix() {
        assert_eq!(
            extract_workspace_name_from_title(
                "file.md - personal (Workspace) - Visual Studio Code"
            ),
            Some("personal".to_string())
        );
    }

    #[test]
    fn bullet_and_active_editor() {
        assert_eq!(
            extract_workspace_name_from_title(
                "\u{25cf} file.md - personal (Workspace) - Visual Studio Code"
            ),
            Some("personal".to_string())
        );
    }

    #[test]
    fn single_folder_window_without_workspace_suffix() {
        assert_eq!(
            extract_workspace_name_from_title("my-project - Visual Studio Code"),
            None
        );
    }

    #[test]
    fn non_vscode_window() {
        assert_eq!(extract_workspace_name_from_title("Chrome - Google"), None);
    }

    #[test]
    fn empty_title() {
        assert_eq!(extract_workspace_name_from_title(""), None);
    }

    #[test]
    fn empty_workspace_name() {
        assert_eq!(
            extract_workspace_name_from_title(" - Visual Studio Code"),
            None
        );
    }

    #[test]
    fn tolerant_of_extra_spaces_around_workspace_suffix() {
        assert_eq!(
            extract_workspace_name_from_title("personal  (Workspace)  - Visual Studio Code"),
            Some("personal".to_string())
        );
    }

    #[test]
    fn dash_in_workspace_display_name() {
        // "my-proj" has no " - " separator inside, so after stripping suffixes
        // the entire remaining text is the name.
        assert_eq!(
            extract_workspace_name_from_title("my-proj (Workspace) - Visual Studio Code"),
            Some("my-proj".to_string())
        );
    }
}
