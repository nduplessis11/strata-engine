//! Vulkan renderer

use std::ffi::{CStr, CString};

use ash::{Entry, Instance, ext, khr, vk};
use winit::raw_window_handle::{RawDisplayHandle, RawWindowHandle};

use crate::{Result, StrataError};

/// Manages Vulkan rendering state and draw calls
pub struct Renderer {
    _context: VulkanContext,
}

impl Renderer {
    /// Create a new Vulkan renderer
    ///
    /// # Arguments
    ///
    /// * `display_handle` - The raw display handle provided by the windowing system.
    /// * `window_handle` - The raw window handle used to create the Vulkan surface.
    /// * `app_name` - The application name passed to Vulkan for instance identification.
    ///
    /// # Errors
    ///
    /// Returns [`StrataError::RendererInit`] if Vulkan initialization fails
    pub fn new(
        display_handle: RawDisplayHandle,
        window_handle: RawWindowHandle,
        app_name: &str,
    ) -> Result<Self> {
        Ok(Self {
            _context: VulkanContext::new(
                display_handle,
                window_handle,
                app_name,
            )?,
        })
    }
}

/// Contains core Vulkan state information known only by Renderer
struct VulkanContext {
    debug_messenger: Option<vk::DebugUtilsMessengerEXT>,
    debug_utils_loader: Option<ext::debug_utils::Instance>,
    surface: vk::SurfaceKHR,
    surface_loader: khr::surface::Instance,
    _instance: Instance,
    _entry: Entry,
}

impl VulkanContext {
    /// Create a new VulkanContext and initialize Vulkan.
    ///
    /// In debug builds, automatically enables Vulkan validation layers
    /// and sets up a debug messenger for error reporting.
    ///
    /// # Errors
    ///
    /// Returns `StrataError::RendererInit` if Vulkan initialization fails
    pub fn new(
        display_handle: RawDisplayHandle,
        window_handle: RawWindowHandle,
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

        let mut extension_names_vec: Vec<*const i8>;
        let extensions_slice = if cfg!(debug_assertions) {
            extension_names_vec = extensions.to_vec();
            extension_names_vec.push(vk::EXT_DEBUG_UTILS_NAME.as_ptr());
            extension_names_vec.as_slice()
        } else {
            extensions
        };

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

        let layers = unsafe { entry.enumerate_instance_layer_properties()? };
        let has_validation = layers.iter().any(|layer| {
            layer
                .layer_name_as_c_str()
                .unwrap()
                .to_str()
                .unwrap()
                == "VK_LAYER_KHRONOS_validation"
        });

        let layer_name_cstr: CString;
        let layer_names: [*const i8; 1];

        let (layer_count, layer_names_ptr) =
            if cfg!(debug_assertions) && has_validation {
                println!("✓ Enabling Vulkan validation layer");
                layer_name_cstr = CString::new("VK_LAYER_KHRONOS_validation")
                    .expect("Layer name must not contain null bytes");
                layer_names = [layer_name_cstr.as_ptr()];
                (1, layer_names.as_ptr())
            } else {
                (0, std::ptr::null())
            };

        let create_info = vk::InstanceCreateInfo {
            p_application_info: &app_info,
            enabled_layer_count: layer_count,
            pp_enabled_layer_names: layer_names_ptr,
            enabled_extension_count: extensions_slice.len() as u32,
            pp_enabled_extension_names: extensions_slice.as_ptr(),
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

        let (debug_utils_loader, debug_messenger) = if cfg!(debug_assertions)
            && has_validation
        {
            let loader = ext::debug_utils::Instance::new(&entry, &instance);

            let messenger_info = vk::DebugUtilsMessengerCreateInfoEXT {
                message_severity: vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                    | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING,
                message_type: vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                    | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                    | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
                pfn_user_callback: Some(debug_callback),
                ..Default::default()
            };

            let messenger = unsafe {
                loader.create_debug_utils_messenger(&messenger_info, None)?
            };

            (Some(loader), Some(messenger))
        } else {
            (None, None)
        };

        let surface_loader = khr::surface::Instance::new(&entry, &instance);

        let surface = unsafe {
            ash_window::create_surface(
                &entry,
                &instance,
                display_handle,
                window_handle,
                None,
            )
            .map_err(|e| {
                StrataError::RendererInit(format!(
                    "Failed to create Vulkan surface: {}",
                    e
                ))
            })?
        };

        Ok(Self {
            debug_messenger,
            debug_utils_loader,
            surface_loader,
            surface,
            _instance: instance,
            _entry: entry,
        })
    }
}

impl Drop for VulkanContext {
    fn drop(&mut self) {
        unsafe {
            self.surface_loader
                .destroy_surface(self.surface, None);

            if let Some(messenger) = self.debug_messenger {
                if let Some(loader) = &self.debug_utils_loader {
                    loader.destroy_debug_utils_messenger(messenger, None);
                }
            }
        }
    }
}

unsafe extern "system" fn debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    _message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut std::ffi::c_void,
) -> vk::Bool32 {
    unsafe {
        let callback_data = &*p_callback_data;
        let message = CStr::from_ptr(callback_data.p_message);

        let severity = if message_severity
            .contains(vk::DebugUtilsMessageSeverityFlagsEXT::ERROR)
        {
            "ERROR"
        } else if message_severity
            .contains(vk::DebugUtilsMessageSeverityFlagsEXT::WARNING)
        {
            "WARNING"
        } else {
            "INFO"
        };

        eprintln!("[VULKAN {}] {:?}", severity, message);
        vk::FALSE
    }
}
