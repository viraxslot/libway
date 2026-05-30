//! Building the tray menu tree from the repository list.

use anyhow::Result;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem, Submenu},
    AppHandle, Wry,
};

use super::{ID_ABOUT_GITHUB, ID_CHECK_NOW, ID_MARK_ALL, ID_QUIT, ID_SETTINGS, REPO_PREFIX};
use crate::db::Repo;
use crate::util::now;

/// Tag bucket name for repositories without any tags.
const UNGROUPED: &str = "Ungrouped";

/// A human "N minutes ago" string for a unix timestamp.
fn relative_time(ts: i64) -> String {
    relative_time_from(now(), ts)
}

/// Pure core of [`relative_time`]: format `ts` relative to a given `now`.
/// A future timestamp (clock skew) clamps to "just now".
fn relative_time_from(now: i64, ts: i64) -> String {
    let secs = (now - ts).max(0);
    if secs < 60 {
        "just now".to_string()
    } else if secs < 3600 {
        format!("{}m ago", secs / 60)
    } else if secs < 86400 {
        format!("{}h ago", secs / 3600)
    } else {
        format!("{}d ago", secs / 86400)
    }
}

/// The non-clickable status line shown at the top of the menu.
fn status_label(repos: &[Repo]) -> String {
    let unseen = repos.iter().filter(|r| r.has_unseen).count();
    let head = match unseen {
        0 => "All up to date".to_string(),
        1 => "1 update".to_string(),
        n => format!("{n} updates"),
    };
    // Oldest successful check across repos, if any.
    let last = repos.iter().filter_map(|r| r.last_checked_at).min();
    match last {
        Some(ts) => format!("{head} · checked {}", relative_time(ts)),
        None => format!("{head} · not checked yet"),
    }
}

/// Label for a single repository entry.
fn repo_label(repo: &Repo) -> String {
    let version = repo.latest_version.as_deref().unwrap_or("…");
    let mark = if repo.has_unseen { " ●" } else { "" };
    format!("{}/{} — {}{}", repo.owner, repo.name, version, mark)
}

/// Append one repository as a clickable item to a menu or submenu.
fn append_repo(app: &AppHandle, menu: &Submenu<Wry>, repo: &Repo) -> Result<()> {
    let id = format!("{REPO_PREFIX}{}", repo.id);
    let item = MenuItem::with_id(app, id, repo_label(repo), true, None::<&str>)?;
    menu.append(&item)?;
    Ok(())
}

/// Collect the sorted set of distinct tags across all repos.
fn distinct_tags(repos: &[Repo]) -> Vec<String> {
    let mut tags: Vec<String> = repos.iter().flat_map(|r| r.tags.clone()).collect();
    tags.sort();
    tags.dedup();
    tags
}

/// Build the menu: status line, the repositories (grouped by tag into
/// submenus when any tags exist, otherwise a flat list), then the actions.
pub(super) fn build_menu(app: &AppHandle, repos: &[Repo], any_unseen: bool) -> Result<Menu<Wry>> {
    let menu = Menu::new(app)?;

    // Status line (disabled = non-clickable).
    let status = MenuItem::with_id(app, "status", status_label(repos), false, None::<&str>)?;
    menu.append(&status)?;
    menu.append(&PredefinedMenuItem::separator(app)?)?;

    let tags = distinct_tags(repos);
    if repos.is_empty() {
        append_empty(app, &menu)?;
    } else if tags.is_empty() {
        append_flat(app, &menu, repos)?;
    } else {
        append_grouped(app, &menu, repos, &tags)?;
    }

    menu.append(&PredefinedMenuItem::separator(app)?)?;
    menu.append(&MenuItem::with_id(
        app,
        ID_CHECK_NOW,
        "Check now",
        true,
        None::<&str>,
    )?)?;
    // Enabled only when there is something to clear.
    menu.append(&MenuItem::with_id(
        app,
        ID_MARK_ALL,
        "Mark all as read",
        any_unseen,
        None::<&str>,
    )?)?;
    menu.append(&MenuItem::with_id(
        app,
        ID_SETTINGS,
        "Settings…",
        true,
        None::<&str>,
    )?)?;
    menu.append(&PredefinedMenuItem::separator(app)?)?;
    menu.append(&about_submenu(app)?)?;
    menu.append(&MenuItem::with_id(
        app,
        ID_QUIT,
        "Quit",
        true,
        None::<&str>,
    )?)?;

    Ok(menu)
}

/// "About" submenu: version, authors and a link to the repository.
fn about_submenu(app: &AppHandle) -> Result<Submenu<Wry>> {
    let about = Submenu::with_id(app, "about", "About", true)?;

    // Version and authors are informational; enabled so they show in the
    // normal text color rather than a dimmed/disabled gray. Clicking them is
    // a no-op (no handler).
    let version = format!("libway v{}", env!("CARGO_PKG_VERSION"));
    about.append(&MenuItem::with_id(
        app,
        "about_version",
        version,
        true,
        None::<&str>,
    )?)?;
    about.append(&MenuItem::with_id(
        app,
        "about_authors",
        // "&&" renders as a literal "&"; a single "&" is treated as a
        // mnemonic accelerator by the native menu and would be hidden.
        "By Alexander Vershinin && Claude",
        true,
        None::<&str>,
    )?)?;
    about.append(&PredefinedMenuItem::separator(app)?)?;
    // The leading ↗ hints that this opens an external page.
    about.append(&MenuItem::with_id(
        app,
        ID_ABOUT_GITHUB,
        "↗ View on GitHub",
        true,
        None::<&str>,
    )?)?;
    Ok(about)
}

/// Placeholder shown when no repositories are tracked.
fn append_empty(app: &AppHandle, menu: &Menu<Wry>) -> Result<()> {
    let empty = MenuItem::with_id(app, "noop", "No repositories", false, None::<&str>)?;
    menu.append(&empty)?;
    Ok(())
}

/// No tags anywhere — a simple flat list of clickable repositories.
fn append_flat(app: &AppHandle, menu: &Menu<Wry>, repos: &[Repo]) -> Result<()> {
    for repo in repos {
        let id = format!("{REPO_PREFIX}{}", repo.id);
        let item = MenuItem::with_id(app, id, repo_label(repo), true, None::<&str>)?;
        menu.append(&item)?;
    }
    Ok(())
}

/// One submenu per tag, plus an "Ungrouped" submenu for untagged repos.
fn append_grouped(
    app: &AppHandle,
    menu: &Menu<Wry>,
    repos: &[Repo],
    tags: &[String],
) -> Result<()> {
    for tag in tags {
        let members: Vec<&Repo> = repos.iter().filter(|r| r.tags.contains(tag)).collect();
        append_group(app, menu, tag, &members)?;
    }
    let untagged: Vec<&Repo> = repos.iter().filter(|r| r.tags.is_empty()).collect();
    if !untagged.is_empty() {
        append_group(app, menu, UNGROUPED, &untagged)?;
    }
    Ok(())
}

/// Append a tag group as a submenu: "tag (count) ●", containing its repos.
fn append_group(app: &AppHandle, menu: &Menu<Wry>, tag: &str, members: &[&Repo]) -> Result<()> {
    let unseen = members.iter().any(|r| r.has_unseen);
    let mark = if unseen { " ●" } else { "" };
    let label = format!("{tag} ({}){mark}", members.len());
    let submenu = Submenu::with_id(app, format!("group:{tag}"), label, true)?;
    for repo in members {
        append_repo(app, &submenu, repo)?;
    }
    menu.append(&submenu)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::db::Repo;

    #[test]
    fn relative_time_buckets() {
        let now = 1_000_000;
        assert_eq!(relative_time_from(now, now), "just now");
        assert_eq!(relative_time_from(now, now - 59), "just now");
        assert_eq!(relative_time_from(now, now - 60), "1m ago");
        assert_eq!(relative_time_from(now, now - 3599), "59m ago");
        assert_eq!(relative_time_from(now, now - 3600), "1h ago");
        assert_eq!(relative_time_from(now, now - 86_399), "23h ago");
        assert_eq!(relative_time_from(now, now - 86_400), "1d ago");
        assert_eq!(relative_time_from(now, now - 3 * 86_400), "3d ago");
    }

    #[test]
    fn relative_time_clamps_future_timestamp() {
        // Clock skew: ts in the future must not produce a negative/garbage value.
        assert_eq!(relative_time_from(1_000, 2_000), "just now");
    }

    #[test]
    fn status_label_pluralizes_and_handles_no_check() {
        // No checks yet → no relative time, deterministic.
        assert_eq!(status_label(&[]), "All up to date · not checked yet");

        let clean = Repo::sample("o", "a"); // sample has no last_checked_at
        assert_eq!(status_label(&[clean]), "All up to date · not checked yet");

        let mut one = Repo::sample("o", "a");
        one.has_unseen = true;
        let head = status_label(std::slice::from_ref(&one));
        assert!(head.starts_with("1 update · "), "got: {head}");

        let mut two = Repo::sample("o", "b");
        two.has_unseen = true;
        let head = status_label(&[one, two]);
        assert!(head.starts_with("2 updates · "), "got: {head}");
    }

    #[test]
    fn repo_label_marks_unseen_and_missing_version() {
        let mut no_version = Repo::sample("cli", "cli");
        no_version.latest_version = None;
        assert_eq!(repo_label(&no_version), "cli/cli — …");

        let mut seen = Repo::sample("cli", "cli");
        seen.latest_version = Some("v1.2.3".into());
        assert_eq!(repo_label(&seen), "cli/cli — v1.2.3");

        let mut unseen = Repo::sample("cli", "cli");
        unseen.latest_version = Some("v1.2.3".into());
        unseen.has_unseen = true;
        assert_eq!(repo_label(&unseen), "cli/cli — v1.2.3 ●");
    }

    #[test]
    fn distinct_tags_sorts_and_dedups() {
        let mut a = Repo::sample("o", "a");
        a.tags = vec!["editors".into(), "build".into()];
        let mut b = Repo::sample("o", "b");
        b.tags = vec!["build".into(), "cli".into()];
        let c = Repo::sample("o", "c"); // sample has no tags

        assert_eq!(
            distinct_tags(&[a, b, c]),
            vec!["build".to_string(), "cli".into(), "editors".into()]
        );
    }

    #[test]
    fn distinct_tags_empty_when_no_tags() {
        let r = Repo::sample("o", "a");
        assert!(distinct_tags(&[r]).is_empty());
    }
}
