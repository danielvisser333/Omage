use std::ffi::CStr;
use ash::{vk::{PhysicalDevice, SurfaceKHR, QueueFlags}, Instance, extensions::khr::Surface, Device};
use ash::extensions::khr::Swapchain;
use ash::vk::{DeviceCreateFlags, DeviceCreateInfo, DeviceQueueCreateFlags, DeviceQueueCreateInfo, PhysicalDeviceFeatures, PhysicalDeviceType, StructureType};
use slog::{Logger, crit, info};

use crate::instance::RenderConfig;

pub unsafe fn get_compatible_devices(logger : &Logger, instance : &Instance, surface_loader : &Surface, surface : SurfaceKHR) -> Vec<PhysicalDevice>{
    let devices = match instance.enumerate_physical_devices(){
        Ok(devices) => {devices}
        Err(error) => {
            crit!(logger, "Failed to get Vulkan devices, {}.", error);
            panic!();
        }
    };
    return devices.iter().filter_map(|device|{
        let mut graphics_support = false;
        let mut compute_support = false;
        let mut surface_support = false;
        for (i, queue_family) in instance.get_physical_device_queue_family_properties(*device).iter().enumerate(){
            graphics_support = graphics_support || queue_family.queue_flags.contains(QueueFlags::GRAPHICS);
            compute_support = compute_support || queue_family.queue_flags.contains(QueueFlags::COMPUTE);
            surface_support = surface_support || surface_loader.get_physical_device_surface_support(*device, i as u32, surface).unwrap() && queue_family.queue_flags.contains(QueueFlags::GRAPHICS);
        }
        if graphics_support && compute_support && surface_support{
            Some(*device)
        }
        else{
            None
        }
    }).collect::<Vec<_>>();
}
pub unsafe fn select_physical_device(logger : &Logger, instance : &Instance, supported_devices : Vec<PhysicalDevice>, config : &RenderConfig) -> PhysicalDevice{
    for device in supported_devices.iter(){
        let device_properties = instance.get_physical_device_properties(*device);
        let device_name = CStr::from_ptr(device_properties.device_name.as_ptr());
        let device_name = device_name.to_str().unwrap();
        if device_name == config.gpu{info!(logger, "[thread#{}]Using selected device {}.", rayon::current_thread_index().unwrap() , device_name);return *device}
    }
    for device in supported_devices.iter() {
        let device_properties = instance.get_physical_device_properties(*device);
        if device_properties.device_type == PhysicalDeviceType::DISCRETE_GPU{return *device}
    }
    return supported_devices[0];
}
pub struct QueueInfo{
    graphics_family : u32,
    compute_family : u32,
    transfer_family : u32,
}
impl QueueInfo{
    pub unsafe fn new(logger : &Logger, instance : &Instance, device : PhysicalDevice) -> Self{
        let mut graphics_family = None;
        let mut compute_family = None;
        let mut transfer_family = None;
        for (i, queue_family) in instance.get_physical_device_queue_family_properties(device).iter().enumerate(){
             if queue_family.queue_flags.contains(QueueFlags::GRAPHICS) && graphics_family.is_none(){graphics_family = Some(i as u32)}
             if queue_family.queue_flags.contains(QueueFlags::COMPUTE) && compute_family.is_none(){compute_family = Some(i as u32)}
             else if queue_family.queue_flags.contains(QueueFlags::COMPUTE) && !queue_family.queue_flags.contains(QueueFlags::GRAPHICS){compute_family = Some(i as u32)}
             if queue_family.queue_flags.contains(QueueFlags::TRANSFER) && transfer_family.is_none(){transfer_family = Some(i as u32)}
            else if queue_family.queue_flags.contains(QueueFlags::TRANSFER) && !queue_family.queue_flags.contains(QueueFlags::COMPUTE) && !queue_family.queue_flags.contains(QueueFlags::GRAPHICS){transfer_family = Some(i as u32)}
        }
        if graphics_family.is_none() || compute_family.is_none() || transfer_family.is_none(){
            crit!(logger, "Failed to assign queue families");
        }
        return Self{
            graphics_family : graphics_family.unwrap(),
            compute_family : compute_family.unwrap(),
            transfer_family : transfer_family.unwrap(),
        }
    }
}
pub unsafe fn create_device(logger : &Logger, instance : &Instance, queue_info : &QueueInfo, device : PhysicalDevice, dedicated_transfer_family : bool, dedicated_compute_family : bool) -> Device{
    let priorities = [1.0];
    let mut device_queue_families = vec![DeviceQueueCreateInfo{
        s_type : StructureType::DEVICE_QUEUE_CREATE_INFO,
        p_next : std::ptr::null(),
        flags : DeviceQueueCreateFlags::empty(),
        queue_count : 1,
        p_queue_priorities : priorities.as_ptr(),
        queue_family_index : queue_info.graphics_family,
    }];
    if dedicated_compute_family && queue_info.graphics_family != queue_info.compute_family{
        device_queue_families.push(DeviceQueueCreateInfo{
            s_type : StructureType::DEVICE_QUEUE_CREATE_INFO,
            p_next : std::ptr::null(),
            flags : DeviceQueueCreateFlags::empty(),
            queue_count : 1,
            p_queue_priorities : priorities.as_ptr(),
            queue_family_index : queue_info.compute_family,
        });
    }
    if dedicated_transfer_family && queue_info.transfer_family != queue_info.compute_family && queue_info.transfer_family != queue_info.graphics_family{
        device_queue_families.push(DeviceQueueCreateInfo{
            s_type : StructureType::DEVICE_QUEUE_CREATE_INFO,
            p_next : std::ptr::null(),
            flags : DeviceQueueCreateFlags::empty(),
            queue_count : 1,
            p_queue_priorities : priorities.as_ptr(),
            queue_family_index : queue_info.transfer_family,
        });
    }
    let extensions = [Swapchain::name().as_ptr()];
    let features = PhysicalDeviceFeatures::default();
    let device_create_info = DeviceCreateInfo{
        s_type : StructureType::DEVICE_CREATE_INFO,
        p_next : std::ptr::null(),
        flags : DeviceCreateFlags::empty(),
        enabled_extension_count : extensions.len() as u32,
        pp_enabled_extension_names : extensions.as_ptr(),
        enabled_layer_count : 0,
        pp_enabled_layer_names : std::ptr::null(),
        p_queue_create_infos : device_queue_families.as_ptr(),
        queue_create_info_count : device_queue_families.len() as u32,
        p_enabled_features : &features,
    };
    return match instance.create_device(device, &device_create_info, None){
        Ok(device) => {device}
        Err(error) => {
            crit!(logger, "[thread#{}]Failed to create device, {}.", rayon::current_thread_index().unwrap() , error);
            panic!();
        }
    }
}