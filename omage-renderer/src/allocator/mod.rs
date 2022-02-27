use ash::{Device, Instance};
use ash::vk::{DeviceMemory, MemoryPropertyFlags, MemoryRequirements, PhysicalDevice, PhysicalDeviceMemoryProperties};
use slog::Logger;

pub struct Allocator{
    logger : Logger,
    physical_device_memory_properties : PhysicalDeviceMemoryProperties,
    device : Device,
    blocks : Vec<Block>,
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
        for (i, memory_type) in self.physical_device_memory_properties.memory_types.iter().enumerate(){
            if self.is_memory_type_compatible(i as u32, memory_property_flags, type_filter){types.push(i as u32)}
        }
        return types;
    }
    pub fn create_allocation(&mut self, memory_requirements : MemoryRequirements, memory_property_flags : MemoryPropertyFlags) -> Option<Allocation>{

    }
    pub unsafe fn destroy(&self){
        for block in self.blocks.iter(){
            self.device.free_memory(block.memory, None);
        }
    }
}
pub struct Block{
    size : u64,
    memory : DeviceMemory,
    type_index : u32,
}
pub struct Allocation{
    block : usize,
    size : u64,
    offset : u64,
}