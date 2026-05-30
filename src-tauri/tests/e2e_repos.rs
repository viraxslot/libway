//! E2E tests for repository commands (list / add / remove / mark_seen /
//! mark_all_seen / check_now) exercised through the real IPC boundary against
//! an in-memory DB, with the GitHub network layer replaced by a fake.

mod common;

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use common::FakeGitHub;
use libway_lib::db::{self as dbmod, Db, SourceKind};
use serde_json::{json, Value};
use tauri::{Listener, Manager};

#[test]
fn e2e_repos_list_returns_camel_case_contract() {
    let app = common::build_app();
    let window = common::main_window(&app);

    // Seed two repos directly (add_repo hits the network, out of scope).
    common::seed_repo(&app, "cli", "cli", 100);
    common::seed_repo(&app, "BurntSushi", "ripgrep", 200);

    let repos =
        common::invoke(&window, "list_repos", json!({})).expect("list_repos should succeed");

    let arr = repos.as_array().expect("expected a JSON array");
    assert_eq!(arr.len(), 2);

    // Most recently added first (ripgrep has the larger created_at).
    assert_eq!(arr[0]["name"], json!("ripgrep"));
    assert_eq!(arr[1]["owner"], json!("cli"));

    // The frontend contract is camelCase. These keys MUST be present.
    let first = &arr[0];
    assert!(first.get("latestVersion").is_some(), "latestVersion key");
    assert!(first.get("latestUrl").is_some(), "latestUrl key");
    assert!(first.get("sourceKind").is_some(), "sourceKind key");
    assert!(first.get("hasUnseen").is_some(), "hasUnseen key");
    assert!(first.get("lastCheckedAt").is_some(), "lastCheckedAt key");
    assert_eq!(first["hasUnseen"], json!(false));

    // snake_case keys MUST NOT leak to the frontend.
    assert!(
        first.get("has_unseen").is_none(),
        "snake_case must not leak"
    );
    assert!(
        first.get("latest_version").is_none(),
        "snake_case must not leak"
    );

    // Newly added repos have no version yet.
    assert_eq!(first["latestVersion"], Value::Null);
}

#[test]
fn e2e_repos_remove_deletes_and_returns_remaining() {
    let app = common::build_app();
    let window = common::main_window(&app);

    let id = common::seed_repo(&app, "cli", "cli", 100);
    common::seed_repo(&app, "BurntSushi", "ripgrep", 200);

    let remaining = common::invoke(&window, "remove_repo", json!({ "id": id }))
        .expect("remove_repo should succeed");

    let arr = remaining.as_array().expect("expected a JSON array");
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["name"], json!("ripgrep"));
}

#[test]
fn e2e_repos_mark_seen_clears_unseen() {
    let app = common::build_app();
    let window = common::main_window(&app);

    let id = common::seed_repo(&app, "cli", "cli", 100);

    // Give it an unseen update directly (update_version sets has_unseen = 1).
    {
        let db = app.state::<Db>();
        let conn = db.0.lock().unwrap();
        dbmod::update_version(&conn, id, "v1.0.0", "https://x", SourceKind::Release, 5)
            .expect("update_version");
    }

    // Confirm it is unseen via list_repos.
    let before = common::invoke(&window, "list_repos", json!({})).unwrap();
    assert_eq!(before.as_array().unwrap()[0]["hasUnseen"], json!(true));

    // mark_seen returns () (null) on success.
    let res = common::invoke(&window, "mark_seen", json!({ "id": id }))
        .expect("mark_seen should succeed");
    assert_eq!(res, Value::Null);

    let after = common::invoke(&window, "list_repos", json!({})).unwrap();
    assert_eq!(after.as_array().unwrap()[0]["hasUnseen"], json!(false));
}

#[test]
fn e2e_repos_mark_all_seen_clears_every_repo() {
    let app = common::build_app();
    let window = common::main_window(&app);

    let a = common::seed_repo(&app, "a", "a", 100);
    let b = common::seed_repo(&app, "b", "b", 200);
    {
        let db = app.state::<Db>();
        let conn = db.0.lock().unwrap();
        dbmod::update_version(&conn, a, "1", "u", SourceKind::Tag, 5).unwrap();
        dbmod::update_version(&conn, b, "1", "u", SourceKind::Tag, 5).unwrap();
    }

    common::invoke(&window, "mark_all_seen", json!({})).expect("mark_all_seen should succeed");

    let after = common::invoke(&window, "list_repos", json!({})).unwrap();
    for repo in after.as_array().unwrap() {
        assert_eq!(repo["hasUnseen"], json!(false));
    }
}

#[test]
fn e2e_repos_mutating_command_emits_repos_updated() {
    let app = common::build_app();
    let window = common::main_window(&app);

    // Count `repos-updated` events. This is the contract that lets the tray
    // and other windows react to a change without the command knowing about
    // them — the whole point of decoupling commands from the tray.
    let count = Arc::new(AtomicUsize::new(0));
    let counter = count.clone();
    app.listen("repos-updated", move |_event| {
        counter.fetch_add(1, Ordering::SeqCst);
    });

    let id = common::seed_repo(&app, "cli", "cli", 100);

    // A read-only command must NOT emit.
    common::invoke(&window, "list_repos", json!({})).expect("list_repos should succeed");
    assert_eq!(
        count.load(Ordering::SeqCst),
        0,
        "list_repos must not emit repos-updated"
    );

    // A mutating command emits exactly one event.
    common::invoke(&window, "remove_repo", json!({ "id": id }))
        .expect("remove_repo should succeed");
    assert_eq!(
        count.load(Ordering::SeqCst),
        1,
        "remove_repo must emit exactly one repos-updated"
    );
}

#[test]
fn e2e_repos_add_inserts_when_repo_exists() {
    let fake_github = FakeGitHub {
        exists: Ok(true),
        ..Default::default()
    };
    let app = common::build_app_with_github(fake_github);
    let window = common::main_window(&app);

    let repos = common::invoke(&window, "add_repo", json!({ "fullName": "cli/cli" }))
        .expect("add_repo should succeed when the repo exists");

    let arr = repos.as_array().expect("expected a JSON array");
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["owner"], json!("cli"));
    assert_eq!(arr[0]["name"], json!("cli"));
    assert_eq!(arr[0]["latestVersion"], serde_json::Value::Null);
    assert_eq!(arr[0]["hasUnseen"], json!(false));
}

#[test]
fn e2e_repos_add_rejects_when_repo_not_found() {
    let fake_github = FakeGitHub {
        exists: Ok(false),
        ..Default::default()
    };
    let app = common::build_app_with_github(fake_github);
    let window = common::main_window(&app);

    let err = common::invoke(&window, "add_repo", json!({ "fullName": "no/such" }))
        .expect_err("add_repo must reject a repo that does not exist");
    assert_eq!(err, json!("repository no/such was not found on GitHub"));

    let repos = common::invoke(&window, "list_repos", json!({})).unwrap();
    assert_eq!(repos.as_array().unwrap().len(), 0);
}

#[test]
fn e2e_repos_add_surfaces_verification_error() {
    let fake_github = FakeGitHub {
        exists: Err("network down".into()),
        ..Default::default()
    };
    let app = common::build_app_with_github(fake_github);
    let window = common::main_window(&app);

    let err = common::invoke(&window, "add_repo", json!({ "fullName": "cli/cli" }))
        .expect_err("add_repo must surface a verification error");
    let msg = err.as_str().expect("error should be a string");
    assert!(
        msg.contains("could not verify cli/cli"),
        "unexpected error: {msg}"
    );

    let repos = common::invoke(&window, "list_repos", json!({})).unwrap();
    assert_eq!(repos.as_array().unwrap().len(), 0);
}

#[test]
fn e2e_repos_add_rejects_bad_full_name() {
    let app = common::build_app_with_github(FakeGitHub::default());
    let window = common::main_window(&app);

    let err = common::invoke(&window, "add_repo", json!({ "fullName": "not-a-repo" }))
        .expect_err("add_repo must reject a malformed name");
    assert_eq!(err, json!("expected the format owner/name"));
}

#[test]
fn e2e_repos_check_now_updates_version_and_flags_unseen() {
    let fake_github = FakeGitHub {
        exists: Ok(true),
        latest: Ok((
            "v2.0.0".to_string(),
            "https://github.com/o/a/releases/tag/v2.0.0".to_string(),
            SourceKind::Release,
        )),
    };
    let app = common::build_app_with_github(fake_github);
    let window = common::main_window(&app);

    // Seed a repo directly (no version yet).
    common::seed_repo(&app, "o", "a", 100);

    // check_now fetches via the fake, stores the version, returns the list.
    let repos = common::invoke(&window, "check_now", json!({})).expect("check_now should succeed");

    let arr = repos.as_array().expect("expected a JSON array");
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["latestVersion"], json!("v2.0.0"));
    assert_eq!(arr[0]["sourceKind"], json!("release"));
    assert_eq!(
        arr[0]["latestUrl"],
        json!("https://github.com/o/a/releases/tag/v2.0.0")
    );
    // Freshly discovered version on a previously version-less repo:
    // update_version sets has_unseen = 1.
    assert_eq!(arr[0]["hasUnseen"], json!(true));
}
