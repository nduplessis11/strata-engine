use thiserror::Error;

#[derive(Error, Debug)]
pub enum StrataError {
    #[error("Window creation failed: {0}")]
    WindowCreation(String),

    #[error("Renderer initialization failed: {0}")]
    RendererInit(String),

    #[error("Vulkan error: {0}")]
    Vulkan(#[from] ash::vk::Result),
}

pub type Result<T> = std::result::Result<T, StrataError>;
