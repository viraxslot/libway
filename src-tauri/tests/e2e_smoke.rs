//! Smoke test: confirms a mock app and window can be built via the shared
//! harness. If this fails, every other e2e file fails too.

mod common;

#[test]
fn e2e_smoke_builds_app_and_window() {
    let app = common::build_app();
    let _window = common::main_window(&app);
}
