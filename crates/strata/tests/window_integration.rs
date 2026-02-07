//! Integration tests for window creation and management

use strata::window::WindowManager;

#[test]
fn test_window_manager_integration() {
    // Test that we can create a WindowManager with actual config
    let wm = WindowManager::with_config("Integration Test", 640, 480)
        .expect("Should create WindowManager");

    assert_eq!(wm.title(), "Integration Test");
    assert_eq!(wm.size(), (640, 480));
}

// Note: We can't easilly test actual window creation in CI/headless environments
// So we'll test the configuration and setup, but not the actual window display
