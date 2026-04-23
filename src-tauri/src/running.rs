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
        let trimmed = arg
            .trim_start_matches("--file=")
            .trim_start_matches("--folder=");
        if trimmed.ends_with(".code-workspace") {
            return Some(PathBuf::from(trimmed));
        }
    }
    None
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
