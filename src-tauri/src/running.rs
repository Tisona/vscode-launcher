use std::collections::HashSet;
use std::path::PathBuf;
use sysinfo::{ProcessRefreshKind, RefreshKind, System};

/// Pure helper: extract a `.code-workspace` path from a process's argv.
/// Returns the first argument (as an absolute path when possible) whose
/// filename ends with `.code-workspace`.
pub fn extract_workspace_path_from_args(args: &[String]) -> Option<PathBuf> {
    for arg in args {
        let trimmed = arg.trim_start_matches("--file=").trim_start_matches("--folder=");
        if trimmed.ends_with(".code-workspace") {
            return Some(PathBuf::from(trimmed));
        }
    }
    None
}

pub fn running_workspaces() -> HashSet<PathBuf> {
    let sys = System::new_with_specifics(
        RefreshKind::new().with_processes(ProcessRefreshKind::new()),
    );
    let mut found = HashSet::new();
    for (_pid, proc_) in sys.processes() {
        let args: Vec<String> = proc_.cmd().iter().map(|s| s.to_string_lossy().into_owned()).collect();
        if let Some(ws) = extract_workspace_path_from_args(&args) {
            // Canonicalize when possible, so equality compares equal to scanner paths.
            let canon = std::fs::canonicalize(&ws).unwrap_or(ws);
            found.insert(canon);
        }
    }
    found
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
}
