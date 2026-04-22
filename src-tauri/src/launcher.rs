use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Pure resolver: given candidate absolute paths and the `PATH` env value,
/// return the first that points at an existing file. Used in tests with
/// controlled fixtures; production callers get candidates + PATH from OS.
pub fn resolve_code_binary_from(candidates: &[PathBuf], env_path: Option<&str>) -> Option<PathBuf> {
    for c in candidates {
        if c.is_file() {
            return Some(c.clone());
        }
    }
    let path = env_path?;
    let sep = if cfg!(windows) { ';' } else { ':' };
    let exe_names: &[&str] = if cfg!(windows) {
        &["code.cmd", "code.exe"]
    } else {
        &["code"]
    };
    for dir in path.split(sep) {
        for name in exe_names {
            let candidate = Path::new(dir).join(name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

pub fn default_candidates() -> Vec<PathBuf> {
    let mut out = Vec::new();
    if cfg!(windows) {
        if let Some(localapp) = dirs::data_local_dir() {
            out.push(localapp.join("Programs/Microsoft VS Code/bin/code.cmd"));
        }
        out.push(PathBuf::from(r"C:\Program Files\Microsoft VS Code\bin\code.cmd"));
    } else if cfg!(target_os = "macos") {
        out.push(PathBuf::from(
            "/Applications/Visual Studio Code.app/Contents/Resources/app/bin/code",
        ));
    } else {
        // Linux
        out.push(PathBuf::from("/usr/bin/code"));
        out.push(PathBuf::from("/usr/local/bin/code"));
        out.push(PathBuf::from("/snap/bin/code"));
    }
    out
}

pub fn resolve_code_binary() -> Option<PathBuf> {
    let candidates = default_candidates();
    let env_path = std::env::var("PATH").ok();
    resolve_code_binary_from(&candidates, env_path.as_deref())
}

pub fn open(workspace: &Path) -> std::io::Result<()> {
    let bin = resolve_code_binary().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::NotFound, "VSCode CLI not found")
    })?;
    let mut cmd = Command::new(bin);
    cmd.arg(workspace);
    cmd.stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        unsafe {
            cmd.pre_exec(|| {
                libc::setsid();
                Ok(())
            });
        }
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const DETACHED_PROCESS: u32 = 0x0000_0008;
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
        cmd.creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP);
    }
    cmd.spawn()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::write;
    use tempfile::tempdir;

    #[test]
    fn returns_first_existing_candidate() {
        let dir = tempdir().unwrap();
        let real = dir.path().join("code");
        write(&real, "#!/bin/sh\n").unwrap();
        let missing = dir.path().join("missing");
        let found = resolve_code_binary_from(&[missing, real.clone()], None).unwrap();
        assert_eq!(found, real);
    }

    #[test]
    fn falls_back_to_path_env() {
        let dir = tempdir().unwrap();
        let code_name = if cfg!(windows) { "code.cmd" } else { "code" };
        let on_path = dir.path().join(code_name);
        write(&on_path, if cfg!(windows) { "@echo off\n" } else { "#!/bin/sh\n" }).unwrap();
        let path = dir.path().to_string_lossy().into_owned();
        let found = resolve_code_binary_from(&[], Some(&path)).unwrap();
        assert_eq!(found, on_path);
    }

    #[test]
    fn returns_none_when_nothing_found() {
        let dir = tempdir().unwrap();
        let fake_path = dir.path().to_string_lossy().into_owned();
        let result = resolve_code_binary_from(
            &[dir.path().join("nope1"), dir.path().join("nope2")],
            Some(&fake_path),
        );
        assert_eq!(result, None);
    }
}
