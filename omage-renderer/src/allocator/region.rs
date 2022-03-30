use ash::vk::Handle;

use super::Allocator;

#[derive(Clone, Copy)]
pub struct Region{
    pub offset : u64,
    pub size : u64,
}
impl Allocator{
    pub unsafe fn try_fit_in_block(&mut self, size : u64, block_r : u64, alignment : u64) -> Option<u64>{
        let block = self.blocks.iter().position(|block| block.memory.as_raw() == block_r).unwrap();
        let mut regions = self.blocks[block as usize].regions.clone();
        regions.sort_unstable_by_key(|region| region.offset);
        if regions.len() == 0 && size <= self.blocks[block as usize].size{
            self.blocks[block as usize].regions.push(Region{offset:0, size});
            return Some(0);
        }
        for i in 0..regions.len(){
            if i == 0 && regions[i].offset >= size{
                self.blocks[block as usize].regions.push(Region{offset:0, size});
                return Some(0);
            }
            if i != regions.len() - 1 && i != 0{
                let start_alignment = regions[i - 1].offset + regions[i - 1].size;
                let offset = start_alignment + alignment - (start_alignment % alignment);
                if regions[i].offset >= offset + size{
                    self.blocks[block as usize].regions.push(Region{offset,size});
                    return Some(offset);
                }
            }
            if i == regions.len() - 1{
                let start_alignment = regions[i].offset + regions[i].size;
                let offset = start_alignment + alignment - (start_alignment % alignment);
                if self.blocks[block as usize].size >= offset + size{
                    self.blocks[block as usize].regions.push(Region{offset,size});
                    return Some(offset);
                }
            }
        }
        return None;
    }
}