//! Building the tray menu tree from the repository list.

use anyhow::Result;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem, Submenu},
    AppHandle, Wry,
};

use super::{
    ID_ABOUT_GITHUB, ID_CHECK_NOW, ID_MARK_ALL, ID_QUIT, ID_SELF_UPDATE, ID_SETTINGS, REPO_PREFIX,
};
use crate::db::Repo;
use crate::i18n::{tr, Language};
use crate::util::now;

/// A human "N minutes ago" string for `ts` relative to `now` (both unix
/// seconds). A future timestamp (clock skew) clamps to "just now".
fn relative_time_from(lang: Language, now: i64, ts: i64) -> String {
    let secs = (now - ts).max(0);
    if secs < 60 {
        tr::just_now(lang).to_string()
    } else if secs < 3600 {
        tr::minutes_ago(lang, secs / 60)
    } else if secs < 86400 {
        tr::hours_ago(lang, secs / 3600)
    } else {
        tr::days_ago(lang, secs / 86400)
    }
}

/// The non-clickable status line shown at the top of the menu.
fn status_label(lang: Language, repos: &[Repo]) -> String {
    let unseen = repos.iter().filter(|r| r.has_unseen).count();
    let head = match unseen {
        0 => tr::all_up_to_date(lang).to_string(),
        n => tr::updates_count(lang, n as u64),
    };
    // Most recent successful check across repos — "last activity".
    let last = repos.iter().filter_map(|r| r.last_checked_at).max();
    format!("{head} · {}", status_suffix(lang, last, now()))
}

/// The "· checked …" / "· not checked yet" tail of the status line, given the
/// most recent check timestamp (if any). Pure, so the min/max choice and
/// wording stay testable without the clock.
fn status_suffix(lang: Language, last: Option<i64>, now: i64) -> String {
    match last {
        Some(ts) => tr::checked_ago(lang, &relative_time_from(lang, now, ts)),
        None => tr::not_checked_yet(lang).to_string(),
    }
}

/// Label for a single repository entry.
fn repo_label(repo: &Repo) -> String {
    let version = repo.latest_version.as_deref().unwrap_or("…");
    let mark = if repo.has_unseen { " ●" } else { "" };
    format!("{}/{} — {}{}", repo.owner, repo.name, version, mark)
}

/// Collect the sorted set of distinct tags across all repos.
fn distinct_tags(repos: &[Repo]) -> Vec<String> {
    let mut tags: Vec<String> = repos.iter().flat_map(|r| r.tags.clone()).collect();
    tags.sort();
    tags.dedup();
    tags
}

/// A node in the tray menu, described as plain data with no Tauri handle.
/// `build_menu_model` produces these (pure, unit-testable) and `render_node`
/// turns them into native menu items.
#[derive(Debug, PartialEq, Eq)]
enum MenuNode {
    /// A leaf item. `enabled = false` renders as a non-clickable label.
    Item {
        id: String,
        label: String,
        enabled: bool,
    },
    /// A horizontal divider.
    Separator,
    /// A submenu with its own child items.
    Submenu {
        id: String,
        label: String,
        children: Vec<MenuNode>,
    },
}

impl MenuNode {
    fn item(id: impl Into<String>, label: impl Into<String>, enabled: bool) -> Self {
        MenuNode::Item {
            id: id.into(),
            label: label.into(),
            enabled,
        }
    }

    /// The node's id, or "---" for a separator (which has none).
    #[cfg(test)]
    fn id(&self) -> &str {
        match self {
            MenuNode::Separator => "---",
            MenuNode::Item { id, .. } | MenuNode::Submenu { id, .. } => id,
        }
    }

    /// A submenu's children, or an empty slice for leaves/separators.
    #[cfg(test)]
    fn children(&self) -> &[MenuNode] {
        match self {
            MenuNode::Submenu { children, .. } => children,
            _ => &[],
        }
    }
}

/// A single repository as a clickable leaf item.
fn repo_node(repo: &Repo) -> MenuNode {
    MenuNode::item(format!("{REPO_PREFIX}{}", repo.id), repo_label(repo), true)
}

/// Label for a tag group submenu: "name (count) ●" (● when any member unseen).
fn group_label(name: &str, members: &[&Repo]) -> String {
    let mark = if members.iter().any(|r| r.has_unseen) {
        " ●"
    } else {
        ""
    };
    format!("{name} ({}){mark}", members.len())
}

/// A tag group as a submenu containing its repositories. The tag name is used
/// for both the stable id and the visible label.
fn group_node(tag: &str, members: &[&Repo]) -> MenuNode {
    group_node_with_label(tag, tag, members)
}

/// Like `group_node`, but with a separate stable `id` and visible `name`. Used
/// by the "Ungrouped" bucket, whose id must stay constant while its label is
/// localized.
fn group_node_with_label(id: &str, name: &str, members: &[&Repo]) -> MenuNode {
    MenuNode::Submenu {
        id: format!("group:{id}"),
        label: group_label(name, members),
        children: members.iter().map(|r| repo_node(r)).collect(),
    }
}

/// The repository section: a placeholder when empty, a flat list when no tags
/// exist, otherwise one submenu per tag plus an "Ungrouped" bucket.
fn repo_section(lang: Language, repos: &[Repo]) -> Vec<MenuNode> {
    if repos.is_empty() {
        return vec![MenuNode::item("noop", tr::no_repositories(lang), false)];
    }
    let tags = distinct_tags(repos);
    if tags.is_empty() {
        return repos.iter().map(repo_node).collect();
    }

    let mut nodes: Vec<MenuNode> = tags
        .iter()
        .map(|tag| {
            let members: Vec<&Repo> = repos.iter().filter(|r| r.tags.contains(tag)).collect();
            group_node(tag, &members)
        })
        .collect();
    let untagged: Vec<&Repo> = repos.iter().filter(|r| r.tags.is_empty()).collect();
    if !untagged.is_empty() {
        // The "Ungrouped" bucket keeps a stable id ("group:Ungrouped") so click
        // routing and tests don't depend on the localized label.
        nodes.push(group_node_with_label(
            "Ungrouped",
            tr::ungrouped(lang),
            &untagged,
        ));
    }
    nodes
}

/// The "About" submenu node: version, authors and a link to the repository.
fn about_node(lang: Language) -> MenuNode {
    MenuNode::Submenu {
        id: "about".into(),
        label: tr::about(lang).into(),
        children: vec![
            MenuNode::item(
                "about_version",
                format!("libway v{}", env!("CARGO_PKG_VERSION")),
                true,
            ),
            // "&&" renders as a literal "&"; a single "&" is treated as a
            // mnemonic accelerator by the native menu and would be hidden.
            // A proper name, not localized.
            MenuNode::item("about_authors", "By Alexander Vershinin && Claude", true),
            MenuNode::Separator,
            // The leading ↗ hints that this opens an external page.
            MenuNode::item(ID_ABOUT_GITHUB, tr::view_on_github(lang), true),
        ],
    }
}

/// Describe the whole tray menu as plain data: an optional update notice, the
/// status line, the repository section, then the actions. Pure and
/// unit-testable — no Tauri handle.
fn build_menu_model(
    lang: Language,
    repos: &[Repo],
    any_unseen: bool,
    update: Option<&crate::selfupdate::AvailableUpdate>,
) -> Vec<MenuNode> {
    let mut nodes = Vec::new();

    if let Some(update) = update {
        nodes.push(MenuNode::item(
            ID_SELF_UPDATE,
            tr::update_available(lang, &update.version),
            true,
        ));
        nodes.push(MenuNode::Separator);
    }

    nodes.extend([
        // Status line (disabled = non-clickable).
        MenuNode::item("status", status_label(lang, repos), false),
        MenuNode::Separator,
    ]);

    nodes.extend(repo_section(lang, repos));

    nodes.extend([
        MenuNode::Separator,
        MenuNode::item(ID_CHECK_NOW, tr::check_now(lang), true),
        // Enabled only when there is something to clear.
        MenuNode::item(ID_MARK_ALL, tr::mark_all_as_read(lang), any_unseen),
        MenuNode::item(ID_SETTINGS, tr::settings(lang), true),
        MenuNode::Separator,
        about_node(lang),
        MenuNode::item(ID_QUIT, tr::quit(lang), true),
    ]);

    nodes
}

/// Build the native tray menu from the repository list.
pub(super) fn build_menu(
    app: &AppHandle,
    lang: Language,
    repos: &[Repo],
    any_unseen: bool,
    update: Option<&crate::selfupdate::AvailableUpdate>,
) -> Result<Menu<Wry>> {
    let menu = Menu::new(app)?;
    for node in build_menu_model(lang, repos, any_unseen, update) {
        menu.append(&render_node(app, &node)?)?;
    }
    Ok(menu)
}

/// Build a native item kind from a model node, recursing into submenus.
/// This is the only place that touches Tauri; all structure/labels come from
/// the model, so it stays a mechanical translation with no decisions.
fn render_node(app: &AppHandle, node: &MenuNode) -> Result<tauri::menu::MenuItemKind<Wry>> {
    use tauri::menu::IsMenuItem;
    let kind = match node {
        MenuNode::Separator => PredefinedMenuItem::separator(app)?.kind(),
        MenuNode::Item { id, label, enabled } => {
            MenuItem::with_id(app, id, label, *enabled, None::<&str>)?.kind()
        }
        MenuNode::Submenu {
            id,
            label,
            children,
        } => {
            let submenu = Submenu::with_id(app, id, label, true)?;
            for child in children {
                submenu.append(&render_node(app, child)?)?;
            }
            submenu.kind()
        }
    };
    Ok(kind)
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::db::Repo;

    #[test]
    fn relative_time_buckets() {
        let now = 1_000_000;
        let en = Language::En;
        assert_eq!(relative_time_from(en, now, now), "just now");
        assert_eq!(relative_time_from(en, now, now - 59), "just now");
        assert_eq!(relative_time_from(en, now, now - 60), "1m ago");
        assert_eq!(relative_time_from(en, now, now - 3599), "59m ago");
        assert_eq!(relative_time_from(en, now, now - 3600), "1h ago");
        assert_eq!(relative_time_from(en, now, now - 86_399), "23h ago");
        assert_eq!(relative_time_from(en, now, now - 86_400), "1d ago");
        assert_eq!(relative_time_from(en, now, now - 3 * 86_400), "3d ago");
    }

    #[test]
    fn relative_time_clamps_future_timestamp() {
        // Clock skew: ts in the future must not produce a negative/garbage value.
        assert_eq!(relative_time_from(Language::En, 1_000, 2_000), "just now");
    }

    #[test]
    fn status_label_pluralizes_and_handles_no_check() {
        let en = Language::En;
        // No checks yet → no relative time, deterministic.
        assert_eq!(status_label(en, &[]), "All up to date · not checked yet");

        let clean = Repo::sample("o", "a"); // sample has no last_checked_at
        assert_eq!(
            status_label(en, &[clean]),
            "All up to date · not checked yet"
        );

        let mut one = Repo::sample("o", "a");
        one.has_unseen = true;
        let head = status_label(en, std::slice::from_ref(&one));
        assert!(head.starts_with("1 update · "), "got: {head}");

        let mut two = Repo::sample("o", "b");
        two.has_unseen = true;
        let head = status_label(en, &[one, two]);
        assert!(head.starts_with("2 updates · "), "got: {head}");
    }

    #[test]
    fn status_label_russian_pluralizes() {
        let ru = Language::Ru;
        assert_eq!(status_label(ru, &[]), "Всё обновлено · ещё не проверялось");

        let mut one = Repo::sample("o", "a");
        one.has_unseen = true;
        let head = status_label(ru, std::slice::from_ref(&one));
        assert!(head.starts_with("1 обновление · "), "got: {head}");
    }

    #[test]
    fn status_suffix_picks_most_recent_and_handles_never_checked() {
        let now = 10_000;
        let en = Language::En;
        // "Last activity" = the most recent (max) check across repos: a fresh
        // check wins over an older one, so the suffix reflects 2m, not 3h.
        let last = [now - 3 * 3600, now - 120].into_iter().max();
        assert_eq!(status_suffix(en, last, now), "checked 2m ago");

        // Never checked → explicit wording, no relative time.
        assert_eq!(status_suffix(en, None, now), "not checked yet");
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

    /// A repo with the given id and tags (owner/name are placeholders).
    fn repo_with(id: i64, tags: &[&str]) -> Repo {
        let mut r = Repo::sample("o", "r");
        r.id = id;
        r.tags = tags.iter().map(|t| t.to_string()).collect();
        r
    }

    /// Ids of a list of nodes, for asserting structure/order.
    fn ids(nodes: &[MenuNode]) -> Vec<&str> {
        nodes.iter().map(MenuNode::id).collect()
    }

    /// Find the node with the given id.
    fn find<'a>(nodes: &'a [MenuNode], id: &str) -> &'a MenuNode {
        nodes
            .iter()
            .find(|n| n.id() == id)
            .unwrap_or_else(|| panic!("no node with id {id}"))
    }

    #[test]
    fn group_label_counts_members_and_marks_unseen() {
        let a = Repo::sample("o", "a");
        let mut b = Repo::sample("o", "b");
        assert_eq!(group_label("ci", &[&a, &b]), "ci (2)");

        b.has_unseen = true;
        assert_eq!(group_label("ci", &[&a, &b]), "ci (2) ●");
    }

    #[test]
    fn repo_node_uses_prefixed_id() {
        let mut r = Repo::sample("cli", "cli");
        r.id = 42;
        r.latest_version = Some("v1.0.0".into());
        assert_eq!(
            repo_node(&r),
            MenuNode::item("repo:42", "cli/cli — v1.0.0", true)
        );
    }

    #[test]
    fn repo_section_empty_shows_placeholder() {
        assert_eq!(
            repo_section(Language::En, &[]),
            vec![MenuNode::item("noop", "No repositories", false)]
        );
    }

    #[test]
    fn repo_section_untagged_is_flat_list() {
        // Distinct ids so the order is visible; no tags.
        let repos = vec![repo_with(1, &[]), repo_with(2, &[])];
        let section = repo_section(Language::En, &repos);
        // Flat: one leaf item per repo, no submenus.
        assert_eq!(ids(&section), vec!["repo:1", "repo:2"]);
        assert!(section.iter().all(|n| matches!(n, MenuNode::Item { .. })));
    }

    #[test]
    fn repo_section_groups_by_tag_with_ungrouped_bucket() {
        let repos = vec![
            repo_with(1, &["ci"]),
            repo_with(2, &["build"]),
            repo_with(3, &[]), // untagged
        ];
        let section = repo_section(Language::En, &repos);

        // Sorted tag groups first, then the Ungrouped bucket.
        assert_eq!(
            ids(&section),
            vec!["group:build", "group:ci", "group:Ungrouped"]
        );
        // The Ungrouped bucket holds the one untagged repo.
        let ungrouped = &section[2];
        assert_eq!(ungrouped.id(), "group:Ungrouped");
        assert_eq!(ids(ungrouped.children()), vec!["repo:3"]);
    }

    #[test]
    fn build_menu_model_has_expected_skeleton() {
        let repos = vec![Repo::sample("o", "a")];
        let model = build_menu_model(Language::En, &repos, false, None);

        assert_eq!(
            ids(&model),
            vec![
                "status",
                "---",
                "repo:1", // the single untagged repo, flat
                "---",
                ID_CHECK_NOW,
                ID_MARK_ALL,
                ID_SETTINGS,
                "---",
                "about",
                ID_QUIT,
            ]
        );
    }

    #[test]
    fn build_menu_model_mark_all_enabled_follows_any_unseen() {
        let repos = vec![Repo::sample("o", "a")];

        // The "Mark all as read" item is enabled exactly when any_unseen is set.
        let disabled = build_menu_model(Language::En, &repos, false, None);
        assert_eq!(
            find(&disabled, ID_MARK_ALL),
            &MenuNode::item(ID_MARK_ALL, "Mark all as read", false)
        );

        let enabled = build_menu_model(Language::En, &repos, true, None);
        assert_eq!(
            find(&enabled, ID_MARK_ALL),
            &MenuNode::item(ID_MARK_ALL, "Mark all as read", true)
        );
    }

    #[test]
    fn build_menu_model_prepends_update_item_when_present() {
        let repos = vec![Repo::sample("o", "a")];
        let update = crate::selfupdate::AvailableUpdate {
            version: "v0.4.0".into(),
            url: "https://github.com/viraxslot/libway/releases/tag/v0.4.0".into(),
        };
        let model = build_menu_model(Language::En, &repos, false, Some(&update));

        assert_eq!(model[0].id(), ID_SELF_UPDATE);
        assert_eq!(
            &model[0],
            &MenuNode::item(ID_SELF_UPDATE, "↗ Update available: v0.4.0", true)
        );
        assert_eq!(model[1].id(), "---");
        assert_eq!(model[2].id(), "status");
    }

    #[test]
    fn build_menu_model_omits_update_item_when_absent() {
        let repos = vec![Repo::sample("o", "a")];
        let model = build_menu_model(Language::En, &repos, false, None);
        assert_eq!(model[0].id(), "status");
        assert!(model.iter().all(|n| n.id() != ID_SELF_UPDATE));
    }

    #[test]
    fn about_node_links_to_github() {
        let about = about_node(Language::En);
        assert_eq!(about.id(), "about");
        // The external link item is present with the GitHub id.
        assert!(about.children().iter().any(|c| c.id() == ID_ABOUT_GITHUB));
    }
}
