use ash::Device;
use ash::vk::{DeviceMemory, MemoryAllocateInfo, PhysicalDeviceMemoryProperties, StructureType};
use slog::{Logger, warn};

pub struct Block{
    pub memory : DeviceMemory,
    size : u64,
    memory_type : u32,
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
        });
    }
}