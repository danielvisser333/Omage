use ash::vk::{MemoryRequirements, MemoryPropertyFlags, Handle, Image, Buffer};

use super::Allocator;

pub struct Allocation{
    block : u64,
    region : u64,
}
impl Allocator{
    pub unsafe fn create_allocation(&mut self, memory_requirements : MemoryRequirements, flags : MemoryPropertyFlags) -> Allocation{
        for i in 0..self.blocks.len(){
            if self.memory_properties.memory_types[self.blocks[i].memory_type as usize].property_flags.contains(flags) && (memory_requirements.memory_type_bits & (1 << self.blocks[i].memory.as_raw() as u32)) > 0{
                match self.try_fit_in_block(memory_requirements.size, self.blocks[i].memory.as_raw(), memory_requirements.alignment){
                    Some(region) => {
                        return Allocation{
                            block : self.blocks[i].memory.as_raw(),
                            region,
                        }
                    }
                    None=>{}
                };
            }
        }
        let block = self.create_block(memory_requirements.size, memory_requirements.memory_type_bits, flags);
        let region = self.try_fit_in_block(memory_requirements.size, block, memory_requirements.alignment).unwrap();
        return Allocation{
            block,region,
        }
    }
    pub unsafe fn destroy_allocation(&mut self, allocation : &Allocation){
        let block_id = self.blocks.iter().position(|block| block.memory.as_raw() == allocation.block).unwrap();
        let index = self.blocks[block_id].regions.iter().position(|region| region.offset == allocation.region).unwrap();
        self.blocks[block_id].regions.remove(index);
        if self.blocks[block_id].regions.len() == 0{
            self.destroy_block(allocation.block);
        }
    }
    pub unsafe fn allocate_image_memory(&mut self, image : Image, flags : MemoryPropertyFlags) -> Allocation{
        let memory_requirements = self.device.get_image_memory_requirements(image);
        let allocation = self.create_allocation(memory_requirements, flags);
        let block_id = self.blocks.iter().position(|block| block.memory.as_raw() == allocation.block).unwrap();
        self.device.bind_image_memory(image, self.blocks[block_id].memory, allocation.region).unwrap();
        return allocation;
    }
    pub unsafe fn allocate_buffer_memory(&mut self, buffer : Buffer, flags : MemoryPropertyFlags) -> Allocation{
        let memory_requirements = self.device.get_buffer_memory_requirements(buffer);
        let allocation = self.create_allocation(memory_requirements, flags);
        let block_id = self.blocks.iter().position(|block| block.memory.as_raw() == allocation.block).unwrap();
        self.device.bind_buffer_memory(buffer, self.blocks[block_id].memory, allocation.region).unwrap();
        return allocation;
    }
}