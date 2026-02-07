// Strata Engine - A voxel game engine for Daggerfall-like experiences

pub mod error;
pub mod renderer;
pub mod window;

pub use error::{Result, StrataError};
pub use renderer::Renderer;

/// Trait that games implement to define their logic
pub trait Game {
    /// Called every frame with delta time in seconds
    fn update(&mut self, dt: f64);

    /// Called every frame to render
    fn render(&mut self, renderer: &mut Renderer);
}

pub struct Engine {
    // TODO: will hold window and renderer
}

impl Engine {
    /// Create a new engine instance
    ///
    /// # Example
    /// ```no_run
    /// use strata::Engine;
    /// let engine = Engine::new().unwrap();
    /// ```
    pub fn new() -> Result<Self> {
        todo!("Create engine")
    }

    /// Run the engine with the given game
    pub fn run<G: Game>(self, mut _game: G) -> Result<()> {
        todo!("Run engine loop")
    }
}
