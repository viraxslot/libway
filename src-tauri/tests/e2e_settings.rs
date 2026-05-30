//! E2E tests for settings commands (check interval / check-on-startup)
//! exercised through the real IPC boundary against an in-memory DB.

mod common;

use pretty_assertions::assert_eq;
use serde_json::json;

#[test]
fn e2e_settings_check_interval_roundtrips_and_validates() {
    let app = common::build_app();
    let window = common::main_window(&app);

    // set then get.
    common::invoke(&window, "set_check_interval", json!({ "minutes": 30 }))
        .expect("set_check_interval should succeed");
    let got = common::invoke(&window, "get_check_interval", json!({}))
        .expect("get_check_interval should succeed");
    assert_eq!(got, json!(30));

    // minutes < 1 is rejected with an error string.
    let err = common::invoke(&window, "set_check_interval", json!({ "minutes": 0 }))
        .expect_err("interval 0 must be rejected");
    assert_eq!(err, json!("interval must be at least 1 minute"));
}

#[test]
fn e2e_settings_check_on_startup_roundtrips() {
    let app = common::build_app();
    let window = common::main_window(&app);

    common::invoke(&window, "set_check_on_startup", json!({ "enabled": true }))
        .expect("set_check_on_startup should succeed");
    let got = common::invoke(&window, "get_check_on_startup", json!({}))
        .expect("get_check_on_startup should succeed");
    assert_eq!(got, json!(true));

    common::invoke(&window, "set_check_on_startup", json!({ "enabled": false }))
        .expect("set_check_on_startup should succeed");
    let got = common::invoke(&window, "get_check_on_startup", json!({}))
        .expect("get_check_on_startup should succeed");
    assert_eq!(got, json!(false));
}
