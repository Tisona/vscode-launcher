use std::collections::HashMap;
use std::path::PathBuf;
use sysinfo::{Pid, Process, ProcessesToUpdate, System};

#[derive(Debug, Clone, serde::Serialize)]
pub struct WorkspaceStatus {
    pub path: PathBuf,
    pub cpu: f32, // raw per-process-tree sum; 100.0 = one full core
    pub ram_bytes: u64,
    pub window_count: u32,
}

pub struct Poller {
    sys: System,
}

impl Poller {
    pub fn new() -> Self {
        let mut sys = System::new();
        // First refresh — CPU deltas will be zero on first tick.
        sys.refresh_processes(ProcessesToUpdate::All, true);
        Self { sys }
    }

    pub fn tick(&mut self) -> Vec<WorkspaceStatus> {
        self.sys.refresh_processes(ProcessesToUpdate::All, true);
        let processes: HashMap<Pid, &Process> = self
            .sys
            .processes()
            .iter()
            .map(|(p, pr)| (*p, pr))
            .collect();
        let children = build_children_map(&processes);

        // Main-process detection: any process whose argv contains an arg ending in .code-workspace.
        let mut per_workspace: HashMap<PathBuf, (f32, u64, u32)> = HashMap::new();

        for (pid, proc_) in &processes {
            let args: Vec<String> = proc_
                .cmd()
                .iter()
                .map(|s| s.to_string_lossy().into_owned())
                .collect();
            if let Some(ws) = extract_workspace_path_from_args(&args) {
                let canon = std::fs::canonicalize(&ws).unwrap_or(ws);
                let (cpu, ram) = sum_tree(*pid, &children, &processes);
                let entry = per_workspace.entry(canon).or_insert((0.0, 0, 0));
                entry.0 += cpu;
                entry.1 += ram;
                entry.2 += 1;
            }
        }

        per_workspace
            .into_iter()
            .map(|(path, (cpu, ram, windows))| WorkspaceStatus {
                path,
                cpu,
                ram_bytes: ram,
                window_count: windows,
            })
            .collect()
    }
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
}
