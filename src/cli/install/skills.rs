//! Symlink the repo-local `skills/mosaico` skill into harness skill directories.

use super::config::{claude_detected, home_dir};
use super::InstallOpts;
use anyhow::{Context, Result};
use owo_colors::OwoColorize;
use std::path::{Path, PathBuf};

const SKILL_REL: &str = "skills/mosaico";
const SKILL_MARKER: &str = "name: mosaico";

#[derive(Debug, Clone)]
struct SkillTarget {
    label: &'static str,
    path: PathBuf,
    link: SkillLink,
}

#[derive(Debug, Clone, Copy)]
enum SkillLink {
    RepoSource,
    AgentsSkill,
}

fn agents_skill_path(home: &Path) -> PathBuf {
    home.join(".agents/skills/mosaico")
}

fn skill_targets() -> Result<Vec<SkillTarget>> {
    let home = home_dir()?;
    let agents = agents_skill_path(&home);
    let mut targets = vec![SkillTarget {
        label: "agents",
        path: agents,
        link: SkillLink::RepoSource,
    }];
    if claude_detected()? {
        targets.push(SkillTarget {
            label: "claude",
            path: home.join(".claude/skills/mosaico"),
            link: SkillLink::AgentsSkill,
        });
    }
    Ok(targets)
}

/// Resolve `skills/mosaico` inside the mosaico repo checkout.
fn skill_source_dir() -> Result<PathBuf> {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let candidate = manifest.join(SKILL_REL);
    if is_skill_tree(&candidate) {
        return candidate
            .canonicalize()
            .with_context(|| format!("canonicalizing {}", candidate.display()));
    }

    let mut dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(Path::to_path_buf));
    while let Some(d) = dir {
        let candidate = d.join(SKILL_REL);
        if is_skill_tree(&candidate) {
            return candidate
                .canonicalize()
                .with_context(|| format!("canonicalizing {}", candidate.display()));
        }
        dir = d.parent().map(Path::to_path_buf);
    }

    anyhow::bail!(
        "cannot find {SKILL_REL} in the mosaico repo; run `mosaico install` from a repo checkout"
    );
}

fn is_skill_tree(path: &Path) -> bool {
    path.join("SKILL.md").is_file()
        && std::fs::read_to_string(path.join("SKILL.md"))
            .map(|content| content.contains(SKILL_MARKER))
            .unwrap_or(false)
}

pub(super) fn print_status() -> Result<()> {
    println!("{}", "mosaico skill status".bold());
    let source = skill_source_dir().ok();
    if let Some(src) = &source {
        println!(
            "  {:<8} {}",
            "source".dimmed(),
            src.display().to_string().dimmed()
        );
    }
    for target in skill_targets()? {
        let installed = if is_installed(&target.path, source.as_deref()) {
            "installed".green().to_string()
        } else {
            "-".dimmed().to_string()
        };
        let detail = installed_link_detail(&target.path, source.as_deref())
            .unwrap_or_else(|| target.path.display().to_string());
        println!(
            "  {:<8} {:<10} {}",
            target.label.cyan(),
            installed,
            detail.dimmed()
        );
    }
    Ok(())
}

pub(super) fn selection_label() -> Result<String> {
    let source = skill_source_dir().ok();
    let targets = skill_targets()?;
    let installed = targets
        .iter()
        .filter(|target| is_installed(&target.path, source.as_deref()))
        .count();
    let status = match installed {
        0 => "-".dimmed().to_string(),
        n if n == targets.len() => "installed".green().to_string(),
        _ => "partial".yellow().to_string(),
    };
    let detail = targets
        .iter()
        .map(|target| target.path.display().to_string())
        .collect::<Vec<_>>()
        .join(", ");
    Ok(format!(
        "{:<18} {:<14} {}",
        "mosaico skill".cyan().bold(),
        status,
        detail.dimmed()
    ))
}

pub(super) fn install(opts: &InstallOpts) -> Result<()> {
    let source = skill_source_dir()?;
    let agents = agents_skill_path(&home_dir()?);
    let verb = if opts.uninstall {
        "Uninstalling skill from"
    } else {
        "Linking skill into"
    };
    let flag = if opts.dry_run { " (dry-run)" } else { "" };

    for target in skill_targets()? {
        println!("\n{} {}{flag}", verb.bold(), target.label.cyan().bold());
        if opts.uninstall {
            uninstall_target(&target, opts.dry_run)?;
        } else {
            let link_source = match target.link {
                SkillLink::RepoSource => source.as_path(),
                SkillLink::AgentsSkill => agents.as_path(),
            };
            install_target(&target, link_source, opts.dry_run)?;
        }
    }
    Ok(())
}

fn is_installed(path: &Path, expected_source: Option<&Path>) -> bool {
    let Ok(resolved) = path.canonicalize() else {
        return false;
    };
    if !is_skill_tree(&resolved) {
        return false;
    }
    match expected_source {
        Some(src) => resolved == src.canonicalize().unwrap_or_else(|_| src.to_path_buf()),
        None => true,
    }
}

fn installed_link_detail(path: &Path, expected_source: Option<&Path>) -> Option<String> {
    if !is_installed(path, expected_source) {
        return None;
    }
    if path.is_symlink() {
        let target = std::fs::read_link(path).ok()?;
        Some(format!("{} -> {}", path.display(), target.display()))
    } else {
        let resolved = path.canonicalize().ok()?;
        Some(resolved.display().to_string())
    }
}

fn install_target(target: &SkillTarget, source: &Path, dry_run: bool) -> Result<()> {
    if dry_run {
        println!(
            "  would symlink {} -> {}",
            target.path.display(),
            source.display()
        );
        return Ok(());
    }

    link_skill(&target.path, source)?;
    println!("  linked {} -> {}", target.path.display(), source.display());
    Ok(())
}

fn uninstall_target(target: &SkillTarget, dry_run: bool) -> Result<()> {
    if target.path.symlink_metadata().is_err() {
        println!("  nothing to remove");
        return Ok(());
    }
    if dry_run {
        println!("  would remove {}", target.path.display());
        return Ok(());
    }
    remove_skill_link(&target.path)?;
    println!("  removed {}", target.path.display());
    Ok(())
}

fn link_skill(link: &Path, source: &Path) -> Result<()> {
    if let Some(parent) = link.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating {}", parent.display()))?;
    }
    if link.symlink_metadata().is_ok() {
        remove_skill_link(link)?;
    }
    #[cfg(unix)]
    std::os::unix::fs::symlink(source, link)
        .with_context(|| format!("linking {} -> {}", link.display(), source.display()))?;
    #[cfg(not(unix))]
    anyhow::bail!("skill symlinks are only supported on Unix");
    Ok(())
}

fn remove_skill_link(path: &Path) -> Result<()> {
    let meta = path
        .symlink_metadata()
        .with_context(|| format!("reading {}", path.display()))?;
    if meta.file_type().is_symlink() {
        std::fs::remove_file(path)
            .with_context(|| format!("removing symlink {}", path.display()))?;
        return Ok(());
    }
    if meta.is_dir() {
        std::fs::remove_dir_all(path).with_context(|| format!("removing {}", path.display()))?;
        return Ok(());
    }
    std::fs::remove_file(path).with_context(|| format!("removing {}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_env::EnvGuard;

    fn opts(dry_run: bool, uninstall: bool) -> InstallOpts {
        InstallOpts {
            all: false,
            harness: None,
            dry_run,
            status: false,
            uninstall,
        }
    }

    #[test]
    fn install_symlinks_agents_skill_to_repo_source() {
        let temp = tempfile::tempdir().unwrap();
        let _home = EnvGuard::set("HOME", temp.path());
        let source = skill_source_dir().unwrap();

        install(&opts(false, false)).unwrap();

        let link = temp.path().join(".agents/skills/mosaico");
        assert!(link.is_symlink());
        assert_eq!(link.canonicalize().unwrap(), source);
    }

    #[test]
    fn install_symlinks_claude_skill_when_claude_dir_exists() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(temp.path().join(".claude")).unwrap();
        let _home = EnvGuard::set("HOME", temp.path());
        let source = skill_source_dir().unwrap();

        install(&opts(false, false)).unwrap();

        let link = temp.path().join(".claude/skills/mosaico");
        assert!(link.is_symlink());
        assert_eq!(
            std::fs::read_link(&link).unwrap(),
            temp.path().join(".agents/skills/mosaico")
        );
        assert_eq!(link.canonicalize().unwrap(), source);
    }

    #[test]
    fn uninstall_removes_symlink_only() {
        let temp = tempfile::tempdir().unwrap();
        let _home = EnvGuard::set("HOME", temp.path());
        let source = skill_source_dir().unwrap();

        install(&opts(false, false)).unwrap();
        install(&opts(false, true)).unwrap();

        assert!(!temp.path().join(".agents/skills/mosaico").exists());
        assert!(source.join("SKILL.md").is_file());
    }

    #[test]
    fn install_replaces_stale_symlink_with_repo_source() {
        let temp = tempfile::tempdir().unwrap();
        let stale = temp.path().join("stale-skill");
        std::fs::create_dir_all(&stale).unwrap();
        std::fs::write(stale.join("SKILL.md"), "name: mosaico\nold").unwrap();

        let link = temp.path().join(".agents/skills/mosaico");
        std::fs::create_dir_all(link.parent().unwrap()).unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(&stale, &link).unwrap();

        let _home = EnvGuard::set("HOME", temp.path());
        let source = skill_source_dir().unwrap();
        install(&opts(false, false)).unwrap();

        assert!(link.is_symlink());
        assert_eq!(link.canonicalize().unwrap(), source);
        assert!(stale.join("SKILL.md").is_file());
    }

    #[test]
    fn install_replaces_copied_tree_with_symlink() {
        let temp = tempfile::tempdir().unwrap();
        let link = temp.path().join(".agents/skills/mosaico");
        std::fs::create_dir_all(&link).unwrap();
        std::fs::write(link.join("SKILL.md"), "name: mosaico\nold").unwrap();

        let _home = EnvGuard::set("HOME", temp.path());
        let source = skill_source_dir().unwrap();
        install(&opts(false, false)).unwrap();

        assert!(link.is_symlink());
        assert_eq!(link.canonicalize().unwrap(), source);
    }
}
