// Build and run the Tauri application: a menu-bar utility that tracks GitHub
// release versions, notifies on updates, and shows them in the tray.

mod checker;
pub mod commands;
pub mod db;
pub mod github;
mod keychain;
mod migrations;
mod notify;
mod scheduler;
mod tray;

use tauri::{Manager, WindowEvent};

use db::Db;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .setup(|app| {
            // macOS: behave as a menu-bar accessory — no Dock icon, no app menu.
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            // Open the SQLite database under the app's data directory and put
            // it into managed state so commands and the scheduler share it.
            let data_dir = app.path().app_data_dir()?;
            let db = Db::open(&data_dir.join("libway.db"))?;
            app.manage(db);

            // The production GitHub client. Tests replace this with a fake.
            app.manage(Box::new(github::RealGitHub::new()) as Box<dyn github::GitHubApi>);

            // Ask for notification permission up front (macOS shows nothing
            // otherwise).
            notify::ensure_permission(app.handle());

            // Create the tray icon and build its initial menu.
            tray::create(app.handle())?;

            // Start the periodic background checker.
            scheduler::spawn(app.handle());
            Ok(())
        })
        .on_window_event(|window, event| {
            // Closing the settings window hides it instead of quitting the app;
            // the app keeps living in the menu bar.
            if let WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "settings" {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_repos,
            commands::add_repo,
            commands::remove_repo,
            commands::set_repo_tags,
            commands::rename_tag,
            commands::delete_tag,
            commands::mark_seen,
            commands::mark_all_seen,
            commands::check_now,
            commands::get_check_interval,
            commands::set_check_interval,
            commands::get_check_on_startup,
            commands::set_check_on_startup,
            commands::has_token,
            commands::set_token,
            commands::clear_token,
            commands::get_autostart,
            commands::set_autostart,
            commands::open_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while launching libway");
}
