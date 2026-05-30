//! Shared harness for backend integration tests.
//!
//! Builds a minimal Tauri app on the mock runtime with a real in-memory
//! SQLite database in managed state and the non-network commands registered,
//! so tests can invoke commands through the real IPC boundary.
#![allow(dead_code)]

use async_trait::async_trait;
use libway_lib::db::{self, Db, SourceKind};
use libway_lib::events::Event;
use libway_lib::github::{GitHubApi, LatestVersion};
use serde_json::Value;
use tauri::test::{get_ipc_response, mock_builder, mock_context, noop_assets, INVOKE_KEY};
use tauri::webview::InvokeRequest;
use tauri::{App, Listener, Manager, WebviewWindow, WebviewWindowBuilder};

/// Mock runtime type used throughout the harness.
pub type Runtime = tauri::test::MockRuntime;

/// Build a mock app with a real in-memory DB and the in-scope commands.
pub fn build_app() -> App<Runtime> {
    let app = mock_builder()
        // check_all fires a notification on a newly discovered version; without
        // the plugin registered, that call panics the async command on the mock
        // runtime. The notification itself is a no-op under the mock runtime.
        .plugin(tauri_plugin_notification::init())
        .invoke_handler(tauri::generate_handler![
            libway_lib::commands::list_repos,
            libway_lib::commands::remove_repo,
            libway_lib::commands::set_repo_tags,
            libway_lib::commands::rename_tag,
            libway_lib::commands::delete_tag,
            libway_lib::commands::mark_seen,
            libway_lib::commands::mark_all_seen,
            libway_lib::commands::get_check_interval,
            libway_lib::commands::set_check_interval,
            libway_lib::commands::get_check_on_startup,
            libway_lib::commands::set_check_on_startup,
            libway_lib::commands::add_repo,
            libway_lib::commands::check_now,
        ])
        .build(mock_context(noop_assets()))
        .expect("failed to build mock app");

    let db = Db::open_in_memory().expect("failed to open in-memory db");
    app.manage(db);

    // Keep a no-op listener so emitted `repos:updated` events have a sink,
    // mirroring production where the tray listens.
    app.listen(Event::ReposUpdated.as_str(), |_event| {});

    app
}

/// A scripted GitHub client for tests. Each field decides what the
/// corresponding trait method returns, independent of owner/name.
pub struct FakeGitHub {
    /// What `repo_exists` returns: Ok(bool) or an error message.
    pub exists: Result<bool, String>,
    /// What `fetch_latest` returns: a (version, url, kind) or an error message.
    pub latest: Result<(String, String, SourceKind), String>,
}

impl Default for FakeGitHub {
    fn default() -> Self {
        FakeGitHub {
            exists: Err("FakeGitHub.exists not configured".into()),
            latest: Err("FakeGitHub.latest not configured".into()),
        }
    }
}

#[async_trait]
impl GitHubApi for FakeGitHub {
    async fn repo_exists(&self, _owner: &str, _name: &str) -> anyhow::Result<bool> {
        self.exists.clone().map_err(|m| anyhow::anyhow!(m))
    }

    async fn fetch_latest(&self, _owner: &str, _name: &str) -> anyhow::Result<LatestVersion> {
        let (version, url, source_kind) = self.latest.clone().map_err(|m| anyhow::anyhow!(m))?;
        Ok(LatestVersion {
            version,
            url,
            source_kind,
        })
    }
}

/// Build a mock app whose GitHub client is the provided fake.
pub fn build_app_with_github(fake: FakeGitHub) -> App<Runtime> {
    let app = build_app();
    app.manage(Box::new(fake) as Box<dyn GitHubApi>);
    app
}

/// A webview window is required for get_ipc_response to route an invoke.
pub fn main_window(app: &App<Runtime>) -> WebviewWindow<Runtime> {
    WebviewWindowBuilder::new(app, "main", Default::default())
        .build()
        .expect("failed to build mock window")
}

/// Invoke a command through the IPC boundary and return the parsed JSON body
/// on success, or the error JSON on failure.
pub fn invoke(window: &WebviewWindow<Runtime>, cmd: &str, args: Value) -> Result<Value, Value> {
    let res = get_ipc_response(
        window,
        InvokeRequest {
            cmd: cmd.into(),
            callback: tauri::ipc::CallbackFn(0),
            error: tauri::ipc::CallbackFn(1),
            url: "tauri://localhost".parse().unwrap(),
            body: args.into(),
            headers: Default::default(),
            invoke_key: INVOKE_KEY.to_string(),
        },
    );
    match res {
        Ok(v) => Ok(v.deserialize::<Value>().expect("response was not JSON")),
        // On the error path `get_ipc_response` already yields a `serde_json::Value`.
        Err(v) => Err(v),
    }
}

/// Seed a repository directly through the managed Db (setup; add_repo hits the
/// network and is out of scope). Returns the new row id.
pub fn seed_repo(app: &App<Runtime>, owner: &str, name: &str, now: i64) -> i64 {
    let db = app.state::<Db>();
    db.with(|c| db::add_repo(c, owner, name, now))
        .expect("failed to seed repo")
}

/// Seed tags on a repo directly through the managed Db.
pub fn seed_tags(app: &App<Runtime>, id: i64, tags: &[&str]) {
    let db = app.state::<Db>();
    let owned: Vec<String> = tags.iter().map(|t| t.to_string()).collect();
    db.with(|c| db::set_repo_tags(c, id, &owned))
        .expect("failed to seed tags");
}
