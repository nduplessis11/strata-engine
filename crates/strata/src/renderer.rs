//! Vulkan renderer

use std::ffi::CString;

use ash::{Entry, Instance, vk};
use winit::raw_window_handle::RawDisplayHandle;

use crate::{Result, StrataError};

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
    pub fn new(
        display_handle: RawDisplayHandle,
        app_name: &str,
    ) -> Result<Self> {
        Ok(Self {
            _context: VulkanContext::new(display_handle, app_name)?,
        })
    }
}

/// Contains core Vulkan state information known only by Renderer
struct VulkanContext {
    _entry: Entry,
    _instance: Instance,
}

impl VulkanContext {
    /// Create a new VulkanContext and initialize Vulkan.
    ///
    /// # Errors
    ///
    /// Returns `StrataError::RendererInit` if Vulkan initialization fails
    pub fn new(
        display_handle: RawDisplayHandle,
        app_name: &str,
    ) -> Result<Self> {
        let extensions = ash_window::enumerate_required_extensions(
            display_handle,
        )
        .map_err(|e| {
            StrataError::RendererInit(format!(
                "Failed to enumerate required Vulkan extensions: {}",
                e
            ))
        })?;

        let entry = Entry::linked();

        let app_name_cstr = CString::new(app_name)
            .expect("application name must not contain null bytes");
        let engine_name_cstr = CString::new("Strata")
            .expect("engine name must not contain null bytes");

        let app_info = vk::ApplicationInfo {
            p_application_name: app_name_cstr.as_ptr(),
            application_version: vk::make_api_version(0, 0, 1, 0),
            p_engine_name: engine_name_cstr.as_ptr(),
            api_version: vk::make_api_version(0, 1, 3, 0),
            ..Default::default()
        };

        let create_info = vk::InstanceCreateInfo {
            p_application_info: &app_info,
            enabled_extension_count: extensions.len() as u32,
            pp_enabled_extension_names: extensions.as_ptr(),
            ..Default::default()
        };
        let instance = unsafe {
            entry
                .create_instance(&create_info, None)
                .map_err(|e| {
                    StrataError::RendererInit(format!(
                        "Failed to create Vulkan instance: {}",
                        e
                    ))
                })?
        };

        Ok(Self { _entry: entry, _instance: instance })
    }
}
