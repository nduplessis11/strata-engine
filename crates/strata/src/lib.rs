//! Strata Engine - A voxel game engine for Daggerfall-like experiences
//!
//! This crate provides the core engine functionality for building voxel-based
//! games with a retro aesthetic.

pub mod error;
pub mod renderer;
pub mod window;

pub use error::{Result, StrataError};
pub use renderer::Renderer;
pub use window::WindowManager;

use std::time::Instant;

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::WindowId;

/// Trait that games implement to define their logic.
///
/// Implement this trait to create your game, providing update and render logic
/// that the engine will call each frame.
///
/// # Example
///
/// ```no_run
/// use strata::{Game, Renderer};
///
/// struct MyGame;
///
/// impl Game for MyGame {
///     fn update(&mut self, dt: f64) {
///         // Game logic here
///     }
///     
///     fn render(&mut self, renderer: &mut Renderer) {
///         // Rendering here
///     }
/// }
/// ```
pub trait Game {
    /// Called every frame with delta time in seconds
    fn update(&mut self, dt: f64);

    /// Called every frame to render
    fn render(&mut self, renderer: &mut Renderer);
}

/// The main engine instance that manages the game loop, rendering, and window.
pub struct Engine {
    window_manager: WindowManager,
}

impl Engine {
    /// Create a new engine instance
    ///
    /// # Example
    /// ```no_run
    /// use strata::Engine;
    /// let engine = Engine::new()?;
    /// # Ok::<(), strata::StrataError>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `StrataError::WindowCreation` if the window cannot be created.
    pub fn new() -> Result<Self> {
        Ok(Self { window_manager: WindowManager::new()? })
    }

    /// Run the engine with the given game
    ///
    /// # Example
    /// ```no_run
    /// use strata::{Engine, Game, Renderer};
    ///
    /// struct MyGame;
    /// impl Game for MyGame {
    ///     fn update(&mut self, _dt: f64) {}
    ///     fn render(&mut self, _renderer: &mut Renderer) {}
    /// }
    ///
    /// let engine = Engine::new()?;
    /// engine.run(MyGame)?;
    /// # Ok::<(), strata::StrataError>(())
    /// ```
    pub fn run<G: Game>(self, game: G) -> Result<()> {
        let event_loop = EventLoop::new()
            .map_err(|e| StrataError::WindowCreation(e.to_string()))?;

        let mut app = EngineApp {
            window_manager: self.window_manager,
            renderer: None,
            game,
            last_frame_time: Instant::now(),
        };

        event_loop
            .run_app(&mut app)
            .map_err(|e| StrataError::WindowCreation(e.to_string()))?;

        Ok(())
    }
}

/// Internal application handler that manages the game loop
struct EngineApp<G: Game> {
    window_manager: WindowManager,
    renderer: Option<Renderer>,
    game: G,
    last_frame_time: Instant,
}

impl<G: Game> ApplicationHandler for EngineApp<G> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Create the window
        if let Err(e) = self
            .window_manager
            .create_window(event_loop)
        {
            eprintln!("Failed to create window: {}", e);
            event_loop.exit();
            return;
        }

        // TODO: Initialize renderer here

        // Request initial redraw
        if let Some(window) = self.window_manager.window() {
            window.request_redraw();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                // Calculate deta time
                let dt = self
                    .last_frame_time
                    .elapsed()
                    .as_secs_f64();
                self.last_frame_time = Instant::now();

                // Call game update
                self.game.update(dt);

                // Call game render (once we have a renderer)
                if let Some(ref mut renderer) = self.renderer {
                    self.game.render(renderer);
                }

                // Request next frame
                if let Some(window) = self.window_manager.window() {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }
}
