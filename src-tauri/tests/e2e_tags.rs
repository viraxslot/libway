//! E2E tests for tag commands (set / rename / delete) exercised through the
//! real IPC boundary. The detailed normalization logic is unit-tested in
//! db.rs; these confirm the commands work end-to-end over IPC.

mod common;

use pretty_assertions::assert_eq;
use serde_json::json;

#[test]
fn e2e_tags_set_normalizes() {
    let app = common::build_app();
    let window = common::main_window(&app);

    let id = common::seed_repo(&app, "o", "a", 100);

    // set_repo_tags returns the refreshed list with normalized tags:
    // trimmed, case-insensitive dedup keeping first spelling, sorted.
    let after_set = common::invoke(
        &window,
        "set_repo_tags",
        json!({ "id": id, "tags": [" Build ", "editors", "build"] }),
    )
    .expect("set_repo_tags should succeed");
    let tags = &after_set.as_array().unwrap()[0]["tags"];
    assert_eq!(tags, &json!(["Build", "editors"]));
}

#[test]
fn e2e_tags_rename() {
    let app = common::build_app();
    let window = common::main_window(&app);

    let id = common::seed_repo(&app, "o", "a", 100);
    common::seed_tags(&app, id, &["build", "editors"]);

    // rename_tag "build" -> "ci" (case-insensitive source match).
    let after_rename = common::invoke(
        &window,
        "rename_tag",
        json!({ "from": "build", "to": "ci" }),
    )
    .expect("rename_tag should succeed");
    let tags = &after_rename.as_array().unwrap()[0]["tags"];
    assert_eq!(tags, &json!(["ci", "editors"]));
}

#[test]
fn e2e_tags_delete() {
    let app = common::build_app();
    let window = common::main_window(&app);

    let id = common::seed_repo(&app, "o", "a", 100);
    common::seed_tags(&app, id, &["ci", "editors"]);

    let after_delete = common::invoke(&window, "delete_tag", json!({ "tag": "editors" }))
        .expect("delete_tag should succeed");
    let tags = &after_delete.as_array().unwrap()[0]["tags"];
    assert_eq!(tags, &json!(["ci"]));
}
