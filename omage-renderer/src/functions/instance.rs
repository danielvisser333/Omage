use std::ffi::CString;
use ash::{Entry, Instance};
use ash::vk::{ApplicationInfo, InstanceCreateFlags, InstanceCreateInfo, StructureType};
use slog::{crit, Logger};
use winit::window::Window;

pub unsafe fn create_instance(logger : &Logger, entry : &Entry, debugging : bool, window : &Window) -> Option<Instance>{
    let name = CString::new("omage").unwrap();
    let validation_layer = CString::new("VK_LAYER_KHRONOS_validation").unwrap();
    let enabled_layers = if debugging{vec![validation_layer.as_ptr()]}else{vec![]};
    let window_extensions = match ash_window::enumerate_required_extensions(window) {Ok(layers) => {layers} Err(error) => {
        crit!(logger, "Failed to get Vulkan window extensions, {}." , error);
        panic!();
    }};
    let app_info = ApplicationInfo{
        s_type : StructureType::APPLICATION_INFO,
        p_next : std::ptr::null(),
        api_version : ash::vk::API_VERSION_1_0,
        application_version : 0,
        engine_version : 0,
        p_application_name : name.as_ptr(),
        p_engine_name : name.as_ptr(),
    };
    let instance_create_info = InstanceCreateInfo{
        s_type : StructureType::INSTANCE_CREATE_INFO,
        p_next : std::ptr::null(),
        flags : InstanceCreateFlags::empty(),
        p_application_info : &app_info,
        pp_enabled_layer_names : enabled_layers.as_ptr(),
        enabled_layer_count : enabled_layers.len() as u32,
        pp_enabled_extension_names : window_extensions.as_ptr(),
        enabled_extension_count : window_extensions.len() as u32,
    };
    match entry.create_instance(&instance_create_info, None){
        Ok(instance) => {Some(instance)}
        Err(error) => {
            match error{
                ash::vk::Result::ERROR_LAYER_NOT_PRESENT => {create_instance(logger, entry, debugging, window)}
                _ => {
                    crit!(logger, "[thread#{}]Failed to create Vulkan instance, {}.", rayon::current_thread_index().unwrap() , error);
                    panic!();
                }
            }
        }
    }
}