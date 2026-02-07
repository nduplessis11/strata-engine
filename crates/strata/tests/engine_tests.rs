#[allow(unused_imports)]
use strata::{Engine, Game, Renderer};

#[allow(dead_code)]
struct DummyGame;

impl Game for DummyGame {
    fn update(&mut self, _dt: f64) {}
    fn render(&mut self, _renderer: &mut Renderer) {}
}

#[test]
fn test_engine_creation() {
    // Will test once Engine::new() works
    // let engine = Engine::new();
    // assert!(engine.is_ok());
}
