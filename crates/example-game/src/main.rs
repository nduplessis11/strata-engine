use strata::{Engine, Game, Renderer};

const PRINT_FRAMES: bool = false;

struct ExampleGame {
    frame_count: u64,
}

impl ExampleGame {
    fn new() -> Self {
        Self { frame_count: 0 }
    }
}

impl Game for ExampleGame {
    fn name(&self) -> &str {
        "Example Game"
    }

    fn update(&mut self, dt: f64) {
        self.frame_count += 1;

        if PRINT_FRAMES && self.frame_count % 60 == 0 {
            println!("Frame {}, dt: {:.3}ms", self.frame_count, dt * 1000.0);
        }
    }

    fn render(&mut self, _renderer: &mut Renderer) {
        // TODO: will draw a triangle
    }
}

fn main() -> anyhow::Result<()> {
    let engine = Engine::new()?;
    let game = ExampleGame::new();
    engine.run(game)?;
    Ok(())
}
