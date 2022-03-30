use std::cmp::max;

use ash::Device;
use ash::vk::{DeviceMemory, MemoryAllocateInfo, StructureType, Handle, MemoryPropertyFlags};
use slog::{Logger, warn, crit};
use crate::allocator::region::Region;

use super::Allocator;

pub struct Block{
    pub memory : DeviceMemory,
    pub size : u64,
    pub memory_type : u32,
    pub regions : Vec<Region>,
}
impl Block{
    pub unsafe fn new(logger : &Logger, device : &Device, size : u64, memory_type : u32) -> Option<Self>{
        let allocate_info = MemoryAllocateInfo{
            s_type : StructureType::MEMORY_ALLOCATE_INFO,
            p_next : std::ptr::null(),
            allocation_size : size,
            memory_type_index : memory_type,
        };
        let memory = match device.allocate_memory(&allocate_info, None){
            Ok(memory) => {memory}
            Err(error) => {
                warn!(logger, "[thread#{}]Failed to allocate GPU memory, {}.", rayon::current_thread_index().unwrap(), error);
                return None;
            }
        };
        return Some(Self{
            memory_type,
            memory,
            size,
            regions : vec![],
        });
    }
}
impl Allocator{
    pub unsafe fn destroy_block(&self, block_id : u64){
        self.device.free_memory(self.blocks[self.blocks.iter().position(|block| block.memory.as_raw() == block_id).unwrap()].memory, None);
    }
    pub unsafe fn create_block(&mut self, size : u64, memory_type_filter : u32, memory_property_flags : MemoryPropertyFlags) -> u64{
        for memory_type in self.get_compatible_memory_types(memory_type_filter, memory_property_flags){
            let block = Block::new(&self.logger, &self.device, max(super::MIN_BLOCK_SIZE, size), memory_type);
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