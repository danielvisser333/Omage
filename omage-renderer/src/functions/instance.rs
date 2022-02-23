use std::ffi::CString;
use ash::{Entry, Instance};
use ash::vk::{API_VERSION_1_2, ApplicationInfo, InstanceCreateFlags, InstanceCreateInfo, StructureType};
use slog::{error, Logger, warn};
use winit::window::Window;
use crate::RenderConfig;

pub unsafe fn create_instance(logger : &Logger, entry : &Entry, config : &RenderConfig , debugging : bool, window : &Window) -> Instance{
    let validation_layer = CString::new("VK_LAYER_KHRONOS_validation").unwrap();
    let window_extensions = match ash_window::enumerate_required_extensions(window){
        Ok(extensions) => {extensions}
        Err(error) => {
            error!(logger, "Failed to get window extensions, {}", error);
            panic!();
        }
    };
    let enabled_extensions = window_extensions.iter().map(|ext| ext.as_ptr()).collect::<Vec<_>>();
    let enabled_layers = if debugging{vec!(validation_layer.as_ptr())}else{vec!()};
    let app_name = CString::new(config.app_name.clone()).unwrap();
    let engine_name = CString::new(config.engine_name.clone()).unwrap();
    let app_info = ApplicationInfo{
        s_type : StructureType::APPLICATION_INFO,
        p_next : std::ptr::null(),
        api_version : API_VERSION_1_2,
        engine_version : config.engine_version,
        application_version : config.app_version,
        p_engine_name : engine_name.as_ptr(),
        p_application_name : app_name.as_ptr(),
    };
    let instance_create_info = InstanceCreateInfo{
        s_type : StructureType::INSTANCE_CREATE_INFO,
        p_next : std::ptr::null(),
        flags : InstanceCreateFlags::empty(),
        enabled_extension_count : enabled_extensions.len() as u32,
        pp_enabled_extension_names : enabled_extensions.as_ptr(),
        enabled_layer_count : enabled_layers.len() as u32,
        pp_enabled_layer_names : enabled_layers.as_ptr(),
        p_application_info : &app_info,
    };
    return match entry.create_instance(&instance_create_info, None){
        Ok(instance) => {instance}
        Err(error) => {
            match debugging{
                true => {
                    warn!(logger, "Tried to create vulkan instance with validation and failed, {}",error);
                    create_instance(logger, entry, config, false, window)
                }
                false => {
                    error!(logger, "Failed to create Vulkan instance {}", error);
                    panic!()
                }
            }
        }
    }
}