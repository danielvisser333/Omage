pub mod block;

use ash::{Device, Instance};
use ash::vk::{PhysicalDevice, PhysicalDeviceMemoryProperties};
use slog::Logger;
use crate::allocator::block::Block;

pub struct Allocator{
    logger : Logger,
    device : Device,
    memory_properties : PhysicalDeviceMemoryProperties,
    blocks : Vec<Block>,
}
impl Allocator{
    pub unsafe fn new(logger : &Logger, instance : &Instance, physical_device : PhysicalDevice, device : &Device) -> Self{
        return Self{
            device:device.clone(),
            memory_properties:instance.get_physical_device_memory_properties(physical_device),
            logger : logger.clone(),
            blocks : vec![],
        }
    }
    pub unsafe fn destroy(&mut self){
        for block in self.blocks.iter(){self.device.free_memory(block.memory, None)}
        self.blocks=vec!();
    }
}