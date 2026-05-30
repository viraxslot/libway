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
    let secs = (now() - ts).max(0);
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

    if repos.is_empty() {
        let empty = MenuItem::with_id(app, "noop", "No repositories", false, None::<&str>)?;
        menu.append(&empty)?;
    } else if distinct_tags(repos).is_empty() {
        // No tags anywhere — keep a simple flat list.
        for repo in repos {
            let id = format!("{REPO_PREFIX}{}", repo.id);
            let item = MenuItem::with_id(app, id, repo_label(repo), true, None::<&str>)?;
            menu.append(&item)?;
        }
    } else {
        // One submenu per tag, plus an "Ungrouped" submenu for untagged repos.
        for tag in distinct_tags(repos) {
            let members: Vec<&Repo> = repos.iter().filter(|r| r.tags.contains(&tag)).collect();
            append_group(app, &menu, &tag, &members)?;
        }
        let untagged: Vec<&Repo> = repos.iter().filter(|r| r.tags.is_empty()).collect();
        if !untagged.is_empty() {
            append_group(app, &menu, UNGROUPED, &untagged)?;
        }
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
