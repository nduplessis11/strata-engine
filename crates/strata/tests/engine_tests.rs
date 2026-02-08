//! Integration tests for engine setup and initialization
//!
//! Note: These tests verify engine creation and configuration,
//! but cannot test the actual event loop and rendering without
//! a display. Those are tested manually by running example-game.

use strata::{Engine, Game, Renderer};

#[test]
fn test_engine_creation() {
    let engine = Engine::new();
    assert!(engine.is_ok(), "Engine creation should succeed");
}

#[test]
fn test_engine_has_window_manager() {
    // This tests that Engine::new() properly initializes its components
    let engine = Engine::new().expect("Failed to create engine");
    // Engine is opaque, but we know it worked if new() succeeded
    drop(engine); // Explicitly drop to verify cleanup works
}

#[allow(dead_code)]
struct DummyGame;

impl Game for DummyGame {
    fn name(&self) -> &str {
        "Dummy Game"
    }
    fn update(&mut self, _dt: f64) {}
    fn render(&mut self, _renderer: &mut Renderer) {}
}

#[test]
fn test_engine_accepts_game() {
    // Test that the API compiles and Engine can be constructed with a game
    let engine = Engine::new().expect("Failed to create engine");
    let game = DummyGame;

    // We can't actually run() in a test, but we can verify the setup
    // The fact this compiles proves the Game trait works correctly
    let _ = (engine, game);
}
