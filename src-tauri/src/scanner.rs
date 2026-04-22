use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WorkspaceEntry {
    pub path: PathBuf,
    pub display_name: String,
    pub auto_icon: Option<PathBuf>,
}

const ICON_EXTS: &[&str] = &["png", "svg", "jpg", "jpeg"];

pub fn scan(root: &Path) -> std::io::Result<Vec<WorkspaceEntry>> {
    let mut entries = Vec::new();
    for dirent in std::fs::read_dir(root)? {
        let dirent = dirent?;
        let path = dirent.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("code-workspace") {
            continue;
        }
        let stem = match path.file_stem().and_then(|s| s.to_str()) {
            Some(s) => s.to_owned(),
            None => continue,
        };
        let auto_icon = find_sibling_icon(&path, &stem);
        entries.push(WorkspaceEntry {
            path,
            display_name: stem,
            auto_icon,
        });
    }
    entries.sort_by(|a, b| a.display_name.to_lowercase().cmp(&b.display_name.to_lowercase()));
    Ok(entries)
}

fn find_sibling_icon(workspace: &Path, stem: &str) -> Option<PathBuf> {
    let parent = workspace.parent()?;
    for ext in ICON_EXTS {
        let candidate = parent.join(format!("{}.{}", stem, ext));
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{create_dir, write};
    use tempfile::tempdir;

    #[test]
    fn only_code_workspace_files_are_returned() {
        let dir = tempdir().unwrap();
        write(dir.path().join("a.code-workspace"), "{}").unwrap();
        write(dir.path().join("b.code-workspace"), "{}").unwrap();
        write(dir.path().join("readme.txt"), "x").unwrap();
        write(dir.path().join("other.json"), "{}").unwrap();
        let entries = scan(dir.path()).unwrap();
        let names: Vec<_> = entries.iter().map(|e| e.display_name.clone()).collect();
        assert_eq!(names, vec!["a".to_string(), "b".to_string()]);
    }

    #[test]
    fn scan_is_not_recursive() {
        let dir = tempdir().unwrap();
        write(dir.path().join("top.code-workspace"), "{}").unwrap();
        create_dir(dir.path().join("sub")).unwrap();
        write(dir.path().join("sub/nested.code-workspace"), "{}").unwrap();
        let entries = scan(dir.path()).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].display_name, "top");
    }

    #[test]
    fn auto_icon_matches_same_basename_png() {
        let dir = tempdir().unwrap();
        write(dir.path().join("proj.code-workspace"), "{}").unwrap();
        write(dir.path().join("proj.png"), b"fake png").unwrap();
        let entries = scan(dir.path()).unwrap();
        assert_eq!(entries[0].auto_icon, Some(dir.path().join("proj.png")));
    }

    #[test]
    fn auto_icon_prefers_png_over_jpg() {
        let dir = tempdir().unwrap();
        write(dir.path().join("proj.code-workspace"), "{}").unwrap();
        write(dir.path().join("proj.jpg"), b"x").unwrap();
        write(dir.path().join("proj.png"), b"x").unwrap();
        let entries = scan(dir.path()).unwrap();
        assert_eq!(entries[0].auto_icon.as_ref().unwrap().extension().unwrap(), "png");
    }

    #[test]
    fn auto_icon_none_when_no_sibling_image() {
        let dir = tempdir().unwrap();
        write(dir.path().join("proj.code-workspace"), "{}").unwrap();
        let entries = scan(dir.path()).unwrap();
        assert_eq!(entries[0].auto_icon, None);
    }

    #[test]
    fn missing_root_returns_err() {
        let dir = tempdir().unwrap();
        let missing = dir.path().join("does-not-exist");
        assert!(scan(&missing).is_err());
    }

    #[test]
    fn sort_is_case_insensitive() {
        let dir = tempdir().unwrap();
        write(dir.path().join("Zed.code-workspace"), "{}").unwrap();
        write(dir.path().join("alpha.code-workspace"), "{}").unwrap();
        let entries = scan(dir.path()).unwrap();
        let names: Vec<_> = entries.iter().map(|e| e.display_name.clone()).collect();
        assert_eq!(names, vec!["alpha".to_string(), "Zed".to_string()]);
    }
}
