//! Error types for Strata Engine

use thiserror::Error;

/// Errors that can occur in the Strata engine
#[derive(Error, Debug)]
pub enum StrataError {
    /// Failed to create a window
    #[error("Window creation failed: {0}")]
    WindowCreation(String),

    /// Failed to initialize the Vulkan renderer
    #[error("Renderer initialization failed: {0}")]
    RendererInit(String),

    /// A Vulkan API error occured
    #[error("Vulkan error: {0}")]
    Vulkan(#[from] ash::vk::Result),
}

/// Convenience type alias for Results using StrataError
pub type Result<T> = std::result::Result<T, StrataError>;
