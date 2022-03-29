pub mod block;

use ash::{Device, Instance};
use ash::vk::{PhysicalDevice, PhysicalDeviceMemoryProperties};
use slog::Logger;

pub struct Allocator{
    logger : Logger,
    device : Device,
    memory_properties : PhysicalDeviceMemoryProperties,
}
impl Allocator{
    pub unsafe fn new(logger : &Logger, instance : &Instance, physical_device : PhysicalDevice, device : &Device) -> Self{
        return Self{
            device:device.clone(),
            memory_properties:instance.get_physical_device_memory_properties(physical_device),
            logger : logger.clone(),
        }
    }
    pub unsafe fn destroy(&mut self){

    }
}