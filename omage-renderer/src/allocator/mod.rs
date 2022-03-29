pub mod block;

use ash::{Device, Instance};
use ash::vk::{PhysicalDevice, PhysicalDeviceMemoryProperties};
use crate::allocator::block::Block;

pub struct Allocator{
    device : Device,
    memory_properties : PhysicalDeviceMemoryProperties,
    blocks : Vec<Block>,
}
impl Allocator{
    pub unsafe fn new(instance : &Instance, physical_device : PhysicalDevice, device : &Device) -> Self{
        return Self{
            device:device.clone(),
            memory_properties:instance.get_physical_device_memory_properties(physical_device),
        }
    }
}