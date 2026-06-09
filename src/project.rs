//! Project-slug resolution (M1 §4).
//!
//! Order:
//!   1. `.tenex/project.json` `slug`, searched from cwd upward to the git root.
//!   2. else the git repo name (toplevel basename) — so all worktrees of a repo
//!      share one slug.
//!   3. else the basename of cwd.

use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Deserialize)]
struct ProjectFile {
    slug: Option<String>,
}

/// Resolve the project slug for a working directory.
pub fn resolve(cwd: &Path) -> String {
    let git_root = git_toplevel(cwd);

    // 1. .tenex/project.json with an explicit slug, searching cwd -> git_root
    //    (or cwd -> filesystem root if not in a repo).
    let stop_at = git_root.as_deref();
    if let Some(slug) = find_project_file_slug(cwd, stop_at) {
        return slug;
    }

    // 2. git repo name.
    if let Some(root) = git_root {
        if let Some(name) = basename(&root) {
            return name;
        }
    }

    // 3. basename of cwd.
    basename(cwd).unwrap_or_else(|| "unknown-project".to_string())
}

/// Walk from `start` upward, reading `.tenex/project.json` if present. Stops
/// after processing `stop_at` (inclusive) when given, else at the fs root.
fn find_project_file_slug(start: &Path, stop_at: Option<&Path>) -> Option<String> {
    let mut dir = Some(start.to_path_buf());
    while let Some(d) = dir {
        let candidate = d.join(".tenex").join("project.json");
        if let Some(slug) = read_slug(&candidate) {
            return Some(slug);
        }
        if Some(d.as_path()) == stop_at {
            break;
        }
        dir = d.parent().map(|p| p.to_path_buf());
    }
    None
}

fn read_slug(path: &Path) -> Option<String> {
    let s = std::fs::read_to_string(path).ok()?;
    let pf: ProjectFile = serde_json::from_str(&s).ok()?;
    pf.slug.filter(|s| !s.trim().is_empty())
}

fn git_toplevel(cwd: &Path) -> Option<PathBuf> {
    let out = Command::new("git")
        .arg("-C")
        .arg(cwd)
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8(out.stdout).ok()?;
    let trimmed = s.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(PathBuf::from(trimmed))
    }
}

fn basename(p: &Path) -> Option<String> {
    p.file_name().map(|n| n.to_string_lossy().to_string())
}

/// Testable variant: resolution with no git involvement (git_root = None).
/// Used by unit tests; production `resolve` adds the git step.
#[cfg(test)]
fn resolve_no_git(cwd: &Path) -> String {
    if let Some(slug) = find_project_file_slug(cwd, None) {
        return slug;
    }
    basename(cwd).unwrap_or_else(|| "unknown-project".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_project_file_wins() {
        let dir = tempfile::tempdir().unwrap();
        let tenex = dir.path().join(".tenex");
        std::fs::create_dir_all(&tenex).unwrap();
        std::fs::write(tenex.join("project.json"), r#"{"slug":"my-cool-project"}"#).unwrap();
        assert_eq!(resolve_no_git(dir.path()), "my-cool-project");
    }

    #[test]
    fn project_file_found_walking_upward() {
        let dir = tempfile::tempdir().unwrap();
        let tenex = dir.path().join(".tenex");
        std::fs::create_dir_all(&tenex).unwrap();
        std::fs::write(tenex.join("project.json"), r#"{"slug":"root-slug"}"#).unwrap();
        let nested = dir.path().join("a").join("b").join("c");
        std::fs::create_dir_all(&nested).unwrap();
        assert_eq!(resolve_no_git(&nested), "root-slug");
    }

    #[test]
    fn falls_back_to_basename() {
        let dir = tempfile::tempdir().unwrap();
        let sub = dir.path().join("the-basename");
        std::fs::create_dir_all(&sub).unwrap();
        assert_eq!(resolve_no_git(&sub), "the-basename");
    }

    #[test]
    fn empty_slug_in_file_is_ignored() {
        let dir = tempfile::tempdir().unwrap();
        let tenex = dir.path().join(".tenex");
        std::fs::create_dir_all(&tenex).unwrap();
        std::fs::write(tenex.join("project.json"), r#"{"slug":"  "}"#).unwrap();
        // basename of the tempdir, since the slug is blank.
        let expected = dir
            .path()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();
        assert_eq!(resolve_no_git(dir.path()), expected);
    }
}
