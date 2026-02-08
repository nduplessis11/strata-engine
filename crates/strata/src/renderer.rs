//! Vulkan renderer

use std::ffi::CString;

use ash::{Entry, Instance, vk};
use winit::raw_window_handle::{RawDisplayHandle};

use crate::Result;

/// Manages Vulkan rendering state and draw calls
pub struct Renderer {
    _context: VulkanContext,
}

impl Renderer {
    /// Create a new Vulkan renderer
    ///
    /// # Errors
    ///
    /// Returns `StrataError::RendererInit` if Vulkan initialization fails
    pub fn new(display_handle: RawDisplayHandle, app_name: &str) -> Result<Self> {
        Ok( Self {
            _context: VulkanContext::new(display_handle, app_name)?,
        })
    }
}

struct VulkanContext {
    _entry: Entry,
    _instance: Instance,
}

impl VulkanContext {
    pub fn new(_display_handle: RawDisplayHandle, app_name: &str) -> Result<Self> {
        let entry = Entry::linked();

        let app_name_cstr = CString::new(app_name)
            .expect("application name must not contain null bytes");

        let app_info = vk::ApplicationInfo {
            p_application_name: app_name_cstr.as_ptr(),
            api_version: vk::make_api_version(0, 1, 3, 0),
            ..Default::default()
        };

        let create_info = vk::InstanceCreateInfo {
            p_application_info: &app_info,
            ..Default::default()
        };
        let instance = unsafe { entry.create_instance(&create_info, None)? };

        Ok(Self {
            _entry: entry,
            _instance: instance,
        })
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_renderer_creation() {
        // Will implement once new() works
        // let renderer = Renderer::new();
        // assert!(renderer.is_ok());
    }
}
