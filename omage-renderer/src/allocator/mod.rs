pub mod block;
pub mod region;

use std::cmp::max;
use ash::{Device, Instance};
use ash::vk::{Handle, MemoryPropertyFlags, PhysicalDevice, PhysicalDeviceMemoryProperties};
use slog::{crit, info, Logger};
use crate::allocator::block::Block;

const MIN_BLOCK_SIZE : u64 = 32_000_000;

pub struct Allocator{
    logger : Logger,
    device : Device,
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
        for block in self.blocks.iter(){self.destroy_block(block.memory.as_raw())}
        self.blocks=vec!();
    }
    fn get_compatible_memory_types(&self, filter : u32, flags : MemoryPropertyFlags) -> Vec<u32>{
        let mut compatible_types = vec!();
        for (i, memory_type) in self.memory_properties.memory_types.iter().enumerate(){
            if memory_type.property_flags.contains(flags) && (filter & (1 << i)) > 0{compatible_types.push(i as u32)}
        }
        return compatible_types;
    }
    unsafe fn destroy_block(&self, block_id : u64){
        self.device.free_memory(self.blocks[self.blocks.iter().position(|block| block.memory.as_raw() == block_id).unwrap()].memory, None);
    }
    unsafe fn create_block(&mut self, size : u64, memory_type_filter : u32, memory_property_flags : MemoryPropertyFlags) -> u64{
        for memory_type in self.get_compatible_memory_types(memory_type_filter, memory_property_flags){
            let block = Block::new(&self.logger, &self.device, max(MIN_BLOCK_SIZE, size), memory_type);
            if block.is_some(){
                let block = block.unwrap();
                let block_id = block.memory.as_raw();
                self.blocks.push(block);
                return block_id;
            }
        }
        crit!(self.logger, "[thread#{}]Memory requested that does not exist.", rayon::current_thread_index().unwrap());
        panic!();
    }
}