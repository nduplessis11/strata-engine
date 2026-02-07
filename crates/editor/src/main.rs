use strata::{Engine, Game, Renderer};

struct EditorGame;

impl Game for EditorGame {
    fn update(&mut self, _dt: f64) {
        // Editor game logic
    }

    fn render(&mut self, _renderer: &mut Renderer) {
        // Editor rendering
    }
}

fn main() -> anyhow::Result<()> {
    println!("Strata Editor - Coming soon!");

    let engine = Engine::new()?;
    engine.run(EditorGame)?;
    Ok(())
}
