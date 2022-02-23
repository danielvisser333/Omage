use ash::{Device, Instance};
use ash::extensions::khr::{Surface, Swapchain};
use ash::vk::{DeviceCreateFlags, DeviceCreateInfo, DeviceQueueCreateFlags, DeviceQueueCreateInfo, PhysicalDevice, PhysicalDeviceFeatures, PhysicalDeviceType, QueueFlags, StructureType, SurfaceKHR};
use slog::{error, info, Logger};

pub unsafe fn select_physical_device(logger : &Logger, instance : &Instance, surface_loader : &Surface, surface : SurfaceKHR) -> PhysicalDevice{
    let physical_devices = match instance.enumerate_physical_devices(){
        Ok(devices) => {
            devices
        }
        Err(error) => {
            error!(logger, "Failed to get physical devices, {}", error);
            panic!();
        }
    };
    let mut preferred_device_dedicated = false;
    let mut preferred_device = None;
    for device in physical_devices{
        let mut supports_graphics = false;
        let mut supports_compute = false;
        let mut supports_presentation = false;
        let device_properties = instance.get_physical_device_properties(device);
        for (i, queue_family) in instance.get_physical_device_queue_family_properties(device).iter().enumerate(){
            supports_graphics = supports_graphics || queue_family.queue_flags.contains(QueueFlags::GRAPHICS);
            supports_compute = supports_compute || queue_family.queue_flags.contains(QueueFlags::COMPUTE);
            supports_presentation = supports_presentation || surface_loader.get_physical_device_surface_support(device, i as u32, surface).expect("Failed to check surface support");
        }
        if supports_graphics && supports_compute && supports_presentation{
            info!(logger, "Found compatible GPU: {}", omage_util::get_string_from_slice(&device_properties.device_name));
            if preferred_device == None{
                preferred_device = Some(device);
                preferred_device_dedicated = device_properties.device_type == PhysicalDeviceType::DISCRETE_GPU;
                info!(logger, "Selected {} as the GPU", omage_util::get_string_from_slice(&device_properties.device_name));
            }
            else if !preferred_device_dedicated && device_properties.device_type == PhysicalDeviceType::DISCRETE_GPU{
                preferred_device = Some(device);
                preferred_device_dedicated = device_properties.device_type == PhysicalDeviceType::DISCRETE_GPU;
                info!(logger, "Overwritten GPU to be {}", omage_util::get_string_from_slice(&device_properties.device_name));
            }
            else{
                info!(logger, "Ignored {}", omage_util::get_string_from_slice(&device_properties.device_name));
            }
        }
        else{
            info!(logger, "Found incompatible GPU: {}.", omage_util::get_string_from_slice(&device_properties.device_name));
            info!(logger, "    -Supports: graphics:{}, compute:{}, presentation:{}",supports_graphics,supports_compute,supports_presentation);
        }
    }
    return match preferred_device {
        Some(device) => {device}
        None => {
            error!(logger, "Failed to get Vulkan compatible GPU");
            panic!();
        }
    }
}
pub struct QueueInfo{
    pub graphics_queue : u32,
    pub presentation_queue : u32,
    pub compute_queue : u32,
    pub transfer_queue : u32,
}
impl QueueInfo{
    pub unsafe fn new(instance : &Instance, physical_device : PhysicalDevice, surface_loader : &Surface, surface : SurfaceKHR, logger : &Logger) -> Self{
        let mut graphics_queue = None;
        let mut presentation_queue = None;
        let mut compute_queue = None;
        let mut transfer_queue = None;
        for (i, queue_family) in instance.get_physical_device_queue_family_properties(physical_device).iter().enumerate(){
            if queue_family.queue_flags.contains(QueueFlags::GRAPHICS) && graphics_queue.is_none(){
                graphics_queue = Some(i as u32);
                if surface_loader.get_physical_device_surface_support(physical_device, i as u32, surface).expect("Failed to check Vulkan surface support"){
                    presentation_queue = Some(i as u32);
                }
            }
            if presentation_queue.is_none() && surface_loader.get_physical_device_surface_support(physical_device, i as u32, surface).expect("Failed to check Vulkan surface support"){
                presentation_queue = Some(i as u32);
            }
            if compute_queue.is_none() && queue_family.queue_flags.contains(QueueFlags::COMPUTE){
                compute_queue = Some(i as u32);
            }
            else if queue_family.queue_flags.contains(QueueFlags::COMPUTE) && !queue_family.queue_flags.contains(QueueFlags::GRAPHICS){
                compute_queue = Some(i as u32);
            }
            if transfer_queue.is_none() && queue_family.queue_flags.contains(QueueFlags::TRANSFER){
                transfer_queue = Some(i as u32);
            }
            else if queue_family.queue_flags.contains(QueueFlags::TRANSFER) && !queue_family.queue_flags.contains(QueueFlags::GRAPHICS) && !queue_family.queue_flags.contains(QueueFlags::COMPUTE){
                transfer_queue = Some(i as u32);
            }
        }
        if graphics_queue.is_none() || presentation_queue.is_none() || transfer_queue.is_none() || compute_queue.is_none(){
            error!(logger, "Failed to get queue families");
            panic!();
        };
        let graphics_queue = graphics_queue.unwrap();
        let presentation_queue = presentation_queue.unwrap();
        let compute_queue = compute_queue.unwrap();
        let transfer_queue = transfer_queue.unwrap();

        info!(logger, "    Selected queue family {} as the graphics family", graphics_queue);
        info!(logger, "    Selected queue family {} as the presentation family", presentation_queue);
        info!(logger, "    Selected queue family {} as the transfer family", transfer_queue);
        info!(logger, "    Selected queue family {} as the compute family", compute_queue);

        return Self{
            graphics_queue,
            presentation_queue,
            compute_queue,
            transfer_queue,
        }
    }
}
pub unsafe fn create_device(logger : &Logger, instance : &Instance, physical_device : PhysicalDevice, queue_infos : &QueueInfo) -> Device{
    let priorities = [1.0];
    let mut device_queue_create_infos = vec!(
        DeviceQueueCreateInfo{
            s_type : StructureType::DEVICE_QUEUE_CREATE_INFO,
            p_next : std::ptr::null(),
            flags : DeviceQueueCreateFlags::empty(),
            queue_count : 1,
            p_queue_priorities : priorities.as_ptr(),
            queue_family_index : queue_infos.graphics_queue,
        }
    );
    if queue_infos.graphics_queue != queue_infos.presentation_queue{
        device_queue_create_infos.push(
            DeviceQueueCreateInfo{
                s_type : StructureType::DEVICE_QUEUE_CREATE_INFO,
                p_next : std::ptr::null(),
                flags : DeviceQueueCreateFlags::empty(),
                queue_count : 1,
                p_queue_priorities : priorities.as_ptr(),
                queue_family_index : queue_infos.presentation_queue,
            }
        );
    }
    if queue_infos.graphics_queue != queue_infos.compute_queue && queue_infos.presentation_queue != queue_infos.compute_queue{
        device_queue_create_infos.push(
            DeviceQueueCreateInfo{
                s_type : StructureType::DEVICE_QUEUE_CREATE_INFO,
                p_next : std::ptr::null(),
                flags : DeviceQueueCreateFlags::empty(),
                queue_count : 1,
                p_queue_priorities : priorities.as_ptr(),
                queue_family_index : queue_infos.compute_queue,
            }
        );
    }
    if queue_infos.transfer_queue != queue_infos.graphics_queue && queue_infos.transfer_queue != queue_infos.compute_queue && queue_infos.transfer_queue != queue_infos.presentation_queue{
        device_queue_create_infos.push(
            DeviceQueueCreateInfo{
                s_type : StructureType::DEVICE_QUEUE_CREATE_INFO,
                p_next : std::ptr::null(),
                flags : DeviceQueueCreateFlags::empty(),
                queue_count : 1,
                p_queue_priorities : priorities.as_ptr(),
                queue_family_index : queue_infos.transfer_queue,
            }
        );
    }
    let device_extensions = [Swapchain::name().as_ptr()];
    let device_features = PhysicalDeviceFeatures{
        ..Default::default()
    };
    let device_create_info = DeviceCreateInfo{
        s_type : StructureType::DEVICE_CREATE_INFO,
        p_next : std::ptr::null(),
        flags : DeviceCreateFlags::empty(),
        pp_enabled_layer_names : std::ptr::null(),
        enabled_layer_count : 0,
        enabled_extension_count : device_extensions.len() as u32,
        pp_enabled_extension_names : device_extensions.as_ptr(),
        p_queue_create_infos : device_queue_create_infos.as_ptr(),
        queue_create_info_count : device_queue_create_infos.len() as u32,
        p_enabled_features : &device_features,
    };
    return match instance.create_device(physical_device, &device_create_info, None){
        Ok(device) => {
            info!(logger, "    Successfully created Vulkan device handle");
            device
        }
        Err(error) => {
            error!(logger, "Failed to create Vulkan device handle, {}", error);
            panic!();
        }
    };
}
