use crate::window_enum;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use sysinfo::{Pid, Process, ProcessRefreshKind, ProcessesToUpdate, System};

#[derive(Debug, Clone, serde::Serialize)]
pub struct WorkspaceStatus {
    pub path: PathBuf,
    pub cpu: f32, // raw per-process-tree sum; 100.0 = one full core
    pub ram_bytes: u64,
    pub window_count: u32,
    #[serde(rename = "displayNameHint")]
    pub display_name_hint: Option<String>,
    pub hwnd: Option<i64>,
}

pub struct Poller {
    sys: System,
}

impl Poller {
    pub fn new() -> Self {
        let mut sys = System::new();
        // First refresh — CPU deltas will be zero on first tick. We use
        // `everything()` so `cmd()` is populated (Windows needs this).
        sys.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::everything(),
        );
        Self { sys }
    }

    pub fn tick(&mut self) -> Vec<WorkspaceStatus> {
        self.sys.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::everything(),
        );
        let processes: HashMap<Pid, &Process> = self
            .sys
            .processes()
            .iter()
            .map(|(p, pr)| (*p, pr))
            .collect();
        let children = build_children_map(&processes);

        // Collect main Electron PIDs (no --type=, no wrapper). Record tree
        // metrics and any argv-derived workspace path so we can cross-reference
        // window-title names against real paths.
        struct MainInfo {
            tree_cpu: f32,
            tree_ram: u64,
            argv_workspace: Option<PathBuf>,
        }
        let mut mains: HashMap<Pid, MainInfo> = HashMap::new();

        for (pid, proc_) in &processes {
            let args: Vec<String> = proc_
                .cmd()
                .iter()
                .map(|s| s.to_string_lossy().into_owned())
                .collect();

            // Diagnostic: print every process whose name looks VSCode-ish so we
            // can tell whether sysinfo is capturing command-line args on Windows.
            let name_lc = proc_.name().to_string_lossy().to_lowercase();
            if name_lc.contains("code") {
                eprintln!(
                    "[poller] pid={} name={:?} args_len={} args={:?}",
                    pid,
                    proc_.name(),
                    args.len(),
                    args
                );
            }

            if !is_main_electron(&args) {
                continue;
            }
            let argv_workspace =
                extract_workspace_path_from_args(&args).map(|p| normalize_workspace_path(&p));
            let (cpu, ram) = sum_tree(*pid, &children, &processes);
            mains.insert(
                *pid,
                MainInfo {
                    tree_cpu: cpu,
                    tree_ram: ram,
                    argv_workspace,
                },
            );
        }

        // Enumerate top-level OS windows (no-op on non-Windows).
        let windows = window_enum::enumerate_workspace_windows();
        for w in &windows {
            eprintln!("[poller] window: pid={} name={:?}", w.pid, w.workspace_name);
        }

        // Group windows by PID. For each PID, divide its main's tree metrics
        // by the window count.
        let mut windows_by_pid: HashMap<Pid, Vec<&window_enum::WorkspaceWindow>> = HashMap::new();
        for w in &windows {
            windows_by_pid
                .entry(Pid::from_u32(w.pid))
                .or_default()
                .push(w);
        }

        // Aggregate per-workspace identity across all windows. Identity key:
        //   - resolved PathBuf when argv path is known and its display-name
        //     matches the window's title-derived name
        //   - otherwise an empty PathBuf + display_name_hint (frontend will
        //     resolve name -> real path via the workspaces store)
        //
        // Hash key includes hint so two different name-only workspaces stay
        // separate.
        // Per-workspace aggregation value: (cpu_sum, ram_sum, window_count, hwnd_first).
        type WsAgg = (f32, u64, u32, Option<i64>);
        let mut per_ws: HashMap<(PathBuf, Option<String>), WsAgg> = HashMap::new();

        let mut pids_with_windows: std::collections::HashSet<Pid> =
            std::collections::HashSet::new();

        for (pid, wins) in &windows_by_pid {
            let Some(info) = mains.get(pid) else {
                // Window's owner PID is not a main we recorded (might be a
                // renderer on Windows pre-Electron-20). Skip.
                continue;
            };
            pids_with_windows.insert(*pid);
            let count = wins.len() as f32;
            let share_cpu = info.tree_cpu / count;
            let share_ram = info.tree_ram / wins.len() as u64;

            let argv_name = info
                .argv_workspace
                .as_ref()
                .and_then(|p| workspace_display_name(p));

            for win in wins {
                // If this window's name matches the argv-derived path's display
                // name, we can emit a real path. Otherwise fall back to hint.
                let (path, hint) = match (&info.argv_workspace, &argv_name) {
                    (Some(p), Some(n)) if n == &win.workspace_name => (p.clone(), None),
                    _ => (PathBuf::new(), Some(win.workspace_name.clone())),
                };
                let entry = per_ws.entry((path, hint)).or_insert((0.0, 0, 0, None));
                entry.0 += share_cpu;
                entry.1 += share_ram;
                entry.2 += 1;
                if entry.3.is_none() {
                    entry.3 = Some(win.hwnd);
                }
            }
        }

        // Fallback: main PIDs that have an argv workspace but zero detected
        // windows (non-Windows, or window enumeration missed them). Emit using
        // the legacy argv-based path.
        for (pid, info) in &mains {
            if pids_with_windows.contains(pid) {
                continue;
            }
            let Some(ws) = &info.argv_workspace else {
                continue;
            };
            let entry = per_ws
                .entry((ws.clone(), None))
                .or_insert((0.0, 0, 0, None));
            entry.0 += info.tree_cpu;
            entry.1 += info.tree_ram;
            entry.2 += 1;
        }

        per_ws
            .into_iter()
            .map(|((path, hint), (cpu, ram, count, hwnd))| WorkspaceStatus {
                path,
                cpu,
                ram_bytes: ram,
                window_count: count,
                display_name_hint: hint,
                hwnd,
            })
            .collect()
    }
}

/// True if `args` look like a main Electron (no `--type=`, not a CLI wrapper).
fn is_main_electron(args: &[String]) -> bool {
    if args.is_empty() {
        return false;
    }
    if args.iter().any(|a| a.starts_with("--type=")) {
        return false;
    }
    for arg in args {
        let lc = arg.to_ascii_lowercase();
        if lc.contains("code.cmd") || lc.contains("cli.js") {
            return false;
        }
    }
    true
}

/// Given a path like `…/personal.code-workspace`, return `"personal"`.
fn workspace_display_name(p: &Path) -> Option<String> {
    let file = p.file_name()?.to_string_lossy().into_owned();
    let stripped = file.strip_suffix(".code-workspace")?;
    if stripped.is_empty() {
        return None;
    }
    Some(stripped.to_string())
}

/// Pure helper: given a process map, build `parent_pid -> [child_pids]`.
pub fn build_children_map(processes: &HashMap<Pid, &Process>) -> HashMap<Pid, Vec<Pid>> {
    let mut map: HashMap<Pid, Vec<Pid>> = HashMap::new();
    for (pid, proc_) in processes {
        if let Some(parent) = proc_.parent() {
            map.entry(parent).or_default().push(*pid);
        }
    }
    map
}

/// Sum CPU and RAM across the process tree rooted at `root` (inclusive).
pub fn sum_tree(
    root: Pid,
    children: &HashMap<Pid, Vec<Pid>>,
    processes: &HashMap<Pid, &Process>,
) -> (f32, u64) {
    let mut stack = vec![root];
    let mut cpu = 0.0f32;
    let mut ram = 0u64;
    while let Some(pid) = stack.pop() {
        if let Some(proc_) = processes.get(&pid) {
            cpu += proc_.cpu_usage();
            ram += proc_.memory();
        }
        if let Some(kids) = children.get(&pid) {
            for k in kids {
                stack.push(*k);
            }
        }
    }
    (cpu, ram)
}

pub fn extract_workspace_path_from_args(args: &[String]) -> Option<PathBuf> {
    // Skip Chromium subprocesses (renderer, utility, gpu-process, zygote, …).
    // Only the main Electron process has no `--type=` arg, and that is the
    // one that carries the workspace path.
    if args.iter().any(|a| a.starts_with("--type=")) {
        return None;
    }

    // Skip CLI-wrapper processes (code.cmd, cli.js). On Windows sysinfo
    // sometimes returns the entire command line as a single arg for batch
    // processes, e.g. `Code/bin/code.cmd D:\…\foo.code-workspace`, which
    // ends with `.code-workspace` and would otherwise match as a bogus path.
    for arg in args {
        let lc = arg.to_ascii_lowercase();
        if lc.contains("code.cmd") || lc.contains("cli.js") {
            return None;
        }
    }

    for arg in args {
        // Strip `--file=`, `--folder=`, `--file-uri=`, `--folder-uri=` prefixes
        // (when flag and value are joined with `=`).
        let stripped = arg
            .strip_prefix("--file=")
            .or_else(|| arg.strip_prefix("--folder="))
            .or_else(|| arg.strip_prefix("--file-uri="))
            .or_else(|| arg.strip_prefix("--folder-uri="))
            .unwrap_or(arg);

        if let Some(path) = workspace_arg_to_path(stripped) {
            return Some(path);
        }
    }
    None
}

/// Normalize a Windows path so that running-workspace paths match scanner
/// output: strip the `\\?\` UNC prefix that `std::fs::canonicalize` adds,
/// and uppercase the drive letter. On non-Windows this is a no-op.
fn normalize_workspace_path(p: &Path) -> PathBuf {
    #[cfg(windows)]
    {
        let s = p.to_string_lossy().into_owned();
        let s = s.strip_prefix(r"\\?\").unwrap_or(&s).to_string();
        let bytes = s.as_bytes();
        if bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' {
            let mut out = String::with_capacity(s.len());
            out.push((bytes[0] as char).to_ascii_uppercase());
            out.push_str(&s[1..]);
            return PathBuf::from(out);
        }
        PathBuf::from(s)
    }
    #[cfg(not(windows))]
    {
        p.to_path_buf()
    }
}

/// Convert a single argv string into a workspace path if it refers to a
/// `.code-workspace` file, handling both plain paths and `file://` URIs with
/// percent-encoding (VSCode passes `--file-uri file:///d%3A/.../x.code-workspace`
/// on Windows).
fn workspace_arg_to_path(s: &str) -> Option<PathBuf> {
    if !s.ends_with(".code-workspace") {
        return None;
    }
    if let Some(rest) = s.strip_prefix("file://") {
        return Some(PathBuf::from(decode_file_uri_path(rest)));
    }
    Some(PathBuf::from(s))
}

/// Decode the path portion of a `file://` URI into a native path.
/// Windows URIs look like `file:///d%3A/path/foo` → `d:\path\foo`.
/// POSIX URIs look like `file:///home/u/foo` → `/home/u/foo`.
fn decode_file_uri_path(rest: &str) -> String {
    // On Windows a file URI has `/` before the drive letter; strip it so the
    // decoded path starts with `d:` rather than `/d:`. Heuristic: if the
    // second character after the optional leading slash is `%3A` or `:`, it
    // is a Windows drive-letter URI regardless of the host OS.
    let looks_like_windows = {
        let stripped = rest.strip_prefix('/').unwrap_or(rest);
        let bytes = stripped.as_bytes();
        bytes.len() >= 4
            && bytes[0].is_ascii_alphabetic()
            && (bytes[1] == b':'
                || (bytes[1] == b'%' && bytes[2] == b'3' && (bytes[3] == b'a' || bytes[3] == b'A')))
    };
    let trimmed = if looks_like_windows && rest.starts_with('/') {
        &rest[1..]
    } else {
        rest
    };

    let decoded = percent_decode(trimmed);
    if looks_like_windows {
        decoded.replace('/', "\\")
    } else {
        decoded
    }
}

fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(hi), Some(lo)) = (hex_digit(bytes[i + 1]), hex_digit(bytes[i + 2])) {
                out.push(hi * 16 + lo);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn hex_digit(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_plain_workspace_arg() {
        let args = vec![
            "/usr/bin/code".to_string(),
            "/home/u/projects/alpha.code-workspace".to_string(),
        ];
        assert_eq!(
            extract_workspace_path_from_args(&args),
            Some(PathBuf::from("/home/u/projects/alpha.code-workspace"))
        );
    }

    #[test]
    fn ignores_non_workspace_args() {
        let args = vec![
            "/usr/bin/code".to_string(),
            "--new-window".to_string(),
            "/tmp/regular/file.txt".to_string(),
        ];
        assert_eq!(extract_workspace_path_from_args(&args), None);
    }

    #[test]
    fn strips_file_prefix() {
        let args = vec![
            "Code.exe".to_string(),
            "--file=C:\\users\\u\\proj.code-workspace".to_string(),
        ];
        assert_eq!(
            extract_workspace_path_from_args(&args),
            Some(PathBuf::from("C:\\users\\u\\proj.code-workspace"))
        );
    }

    #[test]
    fn empty_args_returns_none() {
        assert_eq!(extract_workspace_path_from_args(&[]), None);
    }

    #[test]
    fn returns_first_workspace_when_multiple() {
        let args = vec![
            "code".to_string(),
            "/a.code-workspace".to_string(),
            "/b.code-workspace".to_string(),
        ];
        assert_eq!(
            extract_workspace_path_from_args(&args),
            Some(PathBuf::from("/a.code-workspace"))
        );
    }

    #[test]
    fn decodes_windows_file_uri_from_separate_arg() {
        // VSCode on Windows invokes its main process with `--file-uri` and the
        // URI as separate argv entries.
        let args = vec![
            "C:\\Users\\u\\AppData\\Local\\Programs\\Microsoft VS Code\\Code.exe".to_string(),
            "--file-uri".to_string(),
            "file:///d%3A/vault/work/vscode/personal.code-workspace".to_string(),
        ];
        assert_eq!(
            extract_workspace_path_from_args(&args),
            Some(PathBuf::from(
                "d:\\vault\\work\\vscode\\personal.code-workspace"
            ))
        );
    }

    #[test]
    fn decodes_posix_file_uri() {
        let args = vec![
            "code".to_string(),
            "file:///home/u/my%20projects/alpha.code-workspace".to_string(),
        ];
        assert_eq!(
            extract_workspace_path_from_args(&args),
            Some(PathBuf::from("/home/u/my projects/alpha.code-workspace"))
        );
    }

    #[test]
    fn decodes_joined_file_uri_flag() {
        // `--file-uri=<uri>` rather than two separate args.
        let args = vec![
            "code".to_string(),
            "--file-uri=file:///c%3A/proj/x.code-workspace".to_string(),
        ];
        assert_eq!(
            extract_workspace_path_from_args(&args),
            Some(PathBuf::from("c:\\proj\\x.code-workspace"))
        );
    }

    #[test]
    fn chromium_subprocess_is_ignored() {
        // A renderer child of Code.exe: has --type=renderer. Must not match
        // even if an accidental arg resembles a workspace path.
        let args = vec![
            "C:\\Program Files\\Microsoft VS Code\\Code.exe".to_string(),
            "--type=renderer".to_string(),
            "--user-data-dir=C:\\Users\\u\\AppData\\Roaming\\Code".to_string(),
            "D:\\proj\\workspace.code-workspace".to_string(),
        ];
        assert_eq!(extract_workspace_path_from_args(&args), None);
    }

    #[test]
    fn utility_subprocess_is_ignored() {
        let args = vec![
            "Code.exe".to_string(),
            "--type=utility".to_string(),
            "--utility-sub-type=node.mojom.NodeService".to_string(),
        ];
        assert_eq!(extract_workspace_path_from_args(&args), None);
    }

    #[test]
    fn code_cmd_wrapper_is_ignored() {
        // Batch-wrapper process: sysinfo on Windows can return the whole
        // command line as a single arg, ending in `.code-workspace`.
        let args = vec![
            "Code/bin/code.cmd D:\\vault\\work\\vscode\\ci-templates.code-workspace".to_string(),
        ];
        assert_eq!(extract_workspace_path_from_args(&args), None);
    }

    #[test]
    fn cli_js_helper_is_ignored() {
        let args = vec![
            "C:\\Program Files\\Microsoft VS Code\\Code.exe".to_string(),
            "C:\\Program Files\\Microsoft VS Code\\resources\\app\\out\\cli.js".to_string(),
            "D:\\proj\\x.code-workspace".to_string(),
        ];
        assert_eq!(extract_workspace_path_from_args(&args), None);
    }
}
