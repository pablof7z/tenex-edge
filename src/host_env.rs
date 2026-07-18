use std::collections::HashSet;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

/// Build a deterministic executable path for long-lived host processes.
///
/// The daemon can be auto-started by hooks, GUI apps, SSH, or an interactive
/// shell. Those parents expose materially different PATH values on macOS, but
/// hosted harness availability must not depend on which client won the daemon
/// startup race.
pub(crate) fn executable_path(home: &Path, inherited: Option<&OsStr>) -> OsString {
    std::env::join_paths(executable_dirs(home, inherited))
        .unwrap_or_else(|_| OsString::from("/usr/bin:/bin"))
}

/// Resolve a harness executable using the exact same directory policy applied
/// to hosted children. Capability discovery must never advertise a binary that
/// the daemon's launch environment cannot resolve.
pub(crate) fn resolve_executable(
    home: &Path,
    inherited: Option<&OsStr>,
    executable: &str,
) -> Option<PathBuf> {
    executable_dirs(home, inherited)
        .into_iter()
        .map(|dir| dir.join(executable))
        .find(|path| executable_file(path))
}

fn executable_dirs(home: &Path, inherited: Option<&OsStr>) -> Vec<PathBuf> {
    let inherited = inherited
        .into_iter()
        .flat_map(std::env::split_paths)
        .collect::<Vec<_>>();
    let mut candidates = inherited
        .iter()
        .filter(|path| is_nvm_bin(path))
        .cloned()
        .collect::<Vec<_>>();

    candidates.extend([
        home.join(".local/bin"),
        home.join(".cargo/bin"),
        home.join(".bun/bin"),
        home.join(".opencode/bin"),
        home.join(".volta/bin"),
        home.join(".asdf/shims"),
        home.join(".local/share/mise/shims"),
        home.join(".local/share/fnm/aliases/default/bin"),
        home.join("Library/pnpm"),
    ]);
    candidates.extend(nvm_bin_dirs(home));
    candidates.extend([
        PathBuf::from("/opt/homebrew/bin"),
        PathBuf::from("/usr/local/bin"),
    ]);
    candidates.extend(inherited);

    let mut seen = HashSet::new();
    candidates
        .into_iter()
        .filter(|path| seen.insert(path.clone()))
        .collect()
}

pub(crate) fn apply_executable_path(env: &mut Vec<(String, String)>) {
    let Some(home) = std::env::var_os("HOME").filter(|home| !home.is_empty()) else {
        return;
    };
    let path = executable_path(Path::new(&home), std::env::var_os("PATH").as_deref());
    env.retain(|(key, _)| key != "PATH");
    env.push(("PATH".into(), path.to_string_lossy().into_owned()));
}

fn is_nvm_bin(path: &Path) -> bool {
    path.components().any(|part| part.as_os_str() == ".nvm") && path.ends_with("bin")
}

fn nvm_bin_dirs(home: &Path) -> Vec<PathBuf> {
    let versions = home.join(".nvm/versions/node");
    let Ok(entries) = std::fs::read_dir(versions) else {
        return Vec::new();
    };
    let mut bins = entries
        .flatten()
        .map(|entry| entry.path().join("bin"))
        .filter(|path| path.is_dir())
        .collect::<Vec<_>>();
    bins.sort_by(|left, right| right.cmp(left));
    bins
}

fn executable_file(path: &Path) -> bool {
    let Ok(metadata) = std::fs::metadata(path) else {
        return false;
    };
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        metadata.is_file() && metadata.permissions().mode() & 0o111 != 0
    }
    #[cfg(not(unix))]
    {
        metadata.is_file()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn restores_user_and_nvm_bins_to_restricted_service_path() {
        let home = tempfile::tempdir().unwrap();
        let nvm = home.path().join(".nvm/versions/node/v23.11.1/bin");
        std::fs::create_dir_all(&nvm).unwrap();

        let path = executable_path(home.path(), Some(OsStr::new("/usr/bin:/bin")));
        let parts = std::env::split_paths(&path).collect::<Vec<_>>();

        assert!(parts.starts_with(&[
            home.path().join(".local/bin"),
            home.path().join(".cargo/bin"),
            home.path().join(".bun/bin"),
            home.path().join(".opencode/bin"),
        ]));
        assert!(parts.contains(&nvm));
        assert!(parts.contains(&PathBuf::from("/usr/bin")));
    }

    #[test]
    fn preserves_the_callers_active_nvm_version_first() {
        let home = tempfile::tempdir().unwrap();
        let active = home.path().join(".nvm/versions/node/v20/bin");
        let newer = home.path().join(".nvm/versions/node/v23/bin");
        std::fs::create_dir_all(&active).unwrap();
        std::fs::create_dir_all(&newer).unwrap();
        let inherited = std::env::join_paths([Path::new("/usr/bin"), &active]).unwrap();

        let path = executable_path(home.path(), Some(&inherited));

        assert_eq!(std::env::split_paths(&path).next(), Some(active));
    }

    #[test]
    fn resolves_non_default_manager_bins_under_restricted_path() {
        use std::os::unix::fs::PermissionsExt as _;

        let home = tempfile::tempdir().unwrap();
        let bin = home.path().join(".volta/bin");
        std::fs::create_dir_all(&bin).unwrap();
        let codex = bin.join("codex");
        std::fs::write(&codex, "#!/bin/sh\n").unwrap();
        std::fs::set_permissions(&codex, std::fs::Permissions::from_mode(0o755)).unwrap();

        assert_eq!(
            resolve_executable(home.path(), Some(OsStr::new("/usr/bin:/bin")), "codex"),
            Some(codex)
        );
    }
}
