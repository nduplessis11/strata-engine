//! Window management and event handling

use std::sync::Arc;

use winit::dpi::LogicalSize;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes};

use crate::Result;

pub struct WindowManager {
    title: String,
    width: u32,
    height: u32,
    window: Option<Arc<Window>>,
}

impl WindowManager {
    /// Create a new WindowManager with default configuration
    pub fn new() -> Result<Self> {
        Self::with_config("Strata Engine", 800, 600)
    }

    /// Create a WindowManager with custom configuration
    pub fn with_config(title: &str, width: u32, height: u32) -> Result<Self> {
        Ok(Self {
            title: title.to_string(),
            width,
            height,
            window: None, // Window created later in create_window()
        })
    }

    /// Create the actual window (called from event loop's resumed())
    pub fn create_window(
        &mut self,
        event_loop: &ActiveEventLoop,
    ) -> Result<()> {
        let window = event_loop
            .create_window(
                WindowAttributes::default()
                    .with_title(&self.title)
                    .with_inner_size(LogicalSize::new(self.width, self.height)),
            )
            .map_err(|e| crate::StrataError::WindowCreation(e.to_string()))?;

        self.window = Some(Arc::new(window));
        Ok(())
    }

    /// Get the window (if created)
    pub fn window(&self) -> Option<&Arc<Window>> {
        self.window.as_ref()
    }

    /// Get the window title
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Get the window size as (width, height) in pixels
    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_manager_creation() {
        let wm = WindowManager::new();
        assert!(wm.is_ok(), "WindowManager creation should succeed");
    }

    #[test]
    fn test_window_config() {
        let wm = WindowManager::new().expect("Failed to create WindowManager");

        assert_eq!(wm.title(), "Strata Engine");
        assert_eq!(wm.size(), (800, 600));
    }

    #[test]
    fn test_custom_window_config() {
        let wm = WindowManager::with_config("My Game", 1920, 1080)
            .expect("Failed to create WindowManager with custom config");

        assert_eq!(wm.title(), "My Game");
        assert_eq!(wm.size(), (1920, 1080));
    }

    #[test]
    fn test_window_initially_none() {
        let wm = WindowManager::new().expect("Failed to create WindowManager");
        assert!(
            wm.window().is_none(),
            "Window should be None before create_window"
        );
    }
}
