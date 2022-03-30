pub mod block;
pub mod region;
pub mod allocation;

use ash::{Device, Instance};
use ash::vk::{Handle, MemoryPropertyFlags, PhysicalDevice, PhysicalDeviceMemoryProperties};
use slog::{info, Logger};
use crate::allocator::block::Block;

const MIN_BLOCK_SIZE : u64 = 32_000_000;

pub struct Allocator{
    logger : Logger,
    pub device : Device,
    memory_properties : PhysicalDeviceMemoryProperties,
    blocks : Vec<Block>,
}
impl Allocator{
    pub unsafe fn new(logger : &Logger, instance : &Instance, physical_device : PhysicalDevice, device : &Device) -> Self{
        info!(logger, "[thread#{}]Successfully created the Vulkan memory allocator.", rayon::current_thread_index().unwrap());
        return Self{
            device:device.clone(),
            memory_properties:instance.get_physical_device_memory_properties(physical_device),
            logger : logger.clone(),
            blocks : vec![],
        }
    }
    pub unsafe fn destroy(&mut self){
        info!(self.logger, "[thread#{}]Destroying the Vulkan memory allocator.", rayon::current_thread_index().unwrap());
        for i in 0..self.blocks.len(){self.destroy_block(self.blocks[i].memory.as_raw())}
        self.blocks=vec!();
    }
    fn get_compatible_memory_types(&self, filter : u32, flags : MemoryPropertyFlags) -> Vec<u32>{
        let mut compatible_types = vec!();
        for (i, memory_type) in self.memory_properties.memory_types.iter().enumerate(){
            if memory_type.property_flags.contains(flags) && (filter & (1 << i as u32)) > 0{compatible_types.push(i as u32)}
        }
        return compatible_types;
    }
}