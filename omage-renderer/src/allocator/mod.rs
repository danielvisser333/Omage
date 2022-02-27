use ash::{Device, Instance};
use ash::vk::{DeviceMemory, MemoryAllocateInfo, MemoryPropertyFlags, MemoryRequirements, PhysicalDevice, PhysicalDeviceMemoryProperties, Result as VKResult, StructureType};
use slog::{Logger, warn};

pub struct Allocator{
    logger : Logger,
    physical_device_memory_properties : PhysicalDeviceMemoryProperties,
    device : Device,
    blocks : Vec<Option<Block>>,
}
impl Allocator{
    pub unsafe fn new(logger : Logger, instance : &Instance, physical_device : PhysicalDevice, device : &Device) -> Self{
        return Self{
            logger,
            device:device.clone(),
            blocks : vec!(),
            physical_device_memory_properties : instance.get_physical_device_memory_properties(physical_device),
        }
    }
    fn is_memory_type_compatible(&self, index : u32, memory_property_flags : MemoryPropertyFlags, type_filter : u32) -> bool{
        return self.physical_device_memory_properties.memory_types[index as usize].property_flags.contains(memory_property_flags) && (type_filter & (1 << index)) > 0
    }
    fn get_compatible_memory_types(&self, memory_property_flags : MemoryPropertyFlags, type_filter : u32) -> Vec<u32>{
        let mut types = vec!();
        for i in 0..self.physical_device_memory_properties.memory_types.len(){
            if self.is_memory_type_compatible(i as u32, memory_property_flags, type_filter){types.push(i as u32)}
        }
        return types;
    }
    fn fit_block(&mut self, block : Block) -> usize{
        for i in 0..self.blocks.len(){
            if self.blocks[i].is_none(){
                self.blocks[i] = Some(block);
                return i;
            }
        }
        self.blocks.push(Some(block));
        return self.blocks.len() - 1;
    }
    unsafe fn create_block(&mut self, memory_type : usize, size : u64) -> Result<usize, VKResult>{
        let memory_allocate_info = MemoryAllocateInfo{
            s_type : StructureType::MEMORY_ALLOCATE_INFO,
            p_next : std::ptr::null(),
            allocation_size : size,
            memory_type_index : memory_type as u32,
        };
        let memory = match self.device.allocate_memory(&memory_allocate_info, None){
            Ok(memory) => {memory}
            Err(error) => {
                warn!(self.logger, "Failed to allocate memory, size : {}, type : {}, error : {}", size,memory_type,error);
                return Err(error);
            }
        };
        let block = Block{
            regions : vec!(),
            memory,
            memory_type,
            size,
        };
        return Ok(self.fit_block(block));
    }
    unsafe fn destroy_block(&mut self, block_index : usize){
        let block = self.blocks[block_index].as_ref();
        if block.is_none(){
            warn!(self.logger, "Tried to destroy memory block {}, but that block does not exist", block_index);
            return;
        }
        let block = block.unwrap();
        self.device.free_memory(block.memory, None);
        self.blocks[block_index] = None;
    }
    pub unsafe fn destroy(&self){
        for block in self.blocks.iter(){
            if block.is_some(){self.device.free_memory(block.as_ref().unwrap().memory, None)}
        }
    }
}
pub struct Block{
    memory : DeviceMemory,
    memory_type : usize,
    size : u64,
    regions : Vec<Option<Region>>,
}
pub struct Region{
    offset : u64,
    size : u64,
}
pub struct Allocation{
    block : usize,
    region : usize,
}