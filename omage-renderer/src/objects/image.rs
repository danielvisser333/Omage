use ash::{vk::{Image, StructureType, ImageCreateFlags, ImageCreateInfo, SharingMode, ImageUsageFlags, ImageTiling, Extent2D, Format, Extent3D, ImageType, ImageLayout, SampleCountFlags, MemoryPropertyFlags, ImageView, ImageViewCreateInfo, ImageViewCreateFlags, ComponentMapping, ComponentSwizzle, ImageViewType, ImageSubresourceRange, ImageAspectFlags}, Device};
use slog::{Logger, crit};

use crate::allocator::{allocation::Allocation, Allocator};

pub struct AllocatedImage{
    pub image : Image,
    pub allocation : Allocation,
}
pub struct AllocatedImageView{
    pub image : AllocatedImage,
    pub view : ImageView,
}
impl AllocatedImage{
    pub unsafe fn new_2d(logger : &Logger, allocator : &mut Allocator, extent : Extent2D, format : Format, flags : MemoryPropertyFlags, usage : ImageUsageFlags) -> Self{
        let image_create_info = ImageCreateInfo{
            s_type : StructureType::IMAGE_CREATE_INFO,
            p_next : std::ptr::null(),
            flags : ImageCreateFlags::empty(),
            sharing_mode : SharingMode::EXCLUSIVE,
            queue_family_index_count : 0,
            p_queue_family_indices : std::ptr::null(),
            usage,
            tiling : ImageTiling::OPTIMAL,
            array_layers : 1,
            extent : Extent3D{depth:1, width : extent.width, height:extent.height},
            format,
            image_type : ImageType::TYPE_2D,
            initial_layout : ImageLayout::UNDEFINED,
            mip_levels : 1,
            samples : SampleCountFlags::TYPE_1,
        };
        let image = match allocator.device.create_image(&image_create_info, None){
            Ok(image) => {image}
            Err(error) => {
                crit!(logger, "[thread#{}]Failed to create Vulkan image, {}.",rayon::current_thread_index().unwrap(),error);
                panic!();
            }
        };
        let allocation = allocator.allocate_image_memory(image, flags);
        return Self{
            allocation,
            image,
        }
    }
    pub unsafe fn destroy(&self, allocator : &mut Allocator){
        allocator.destroy_allocation(&self.allocation);
        allocator.device.destroy_image(self.image, None);
    }
}
impl AllocatedImageView{
    pub unsafe fn new_2d(logger : &Logger, allocator : &mut Allocator, extent : Extent2D, format : Format, flags : MemoryPropertyFlags, usage : ImageUsageFlags, aspect : ImageAspectFlags) -> Self{
        let image = AllocatedImage::new_2d(logger, allocator, extent, format, flags, usage);
        let image_view_create_info = ImageViewCreateInfo{
            s_type : StructureType::IMAGE_VIEW_CREATE_INFO,
            p_next : std::ptr::null(),
            flags : ImageViewCreateFlags::empty(),
            components : ComponentMapping{r : ComponentSwizzle::R, g : ComponentSwizzle::G, b : ComponentSwizzle::B, a : ComponentSwizzle::A},
            format,
            image:image.image,
            view_type : ImageViewType::TYPE_2D,
            subresource_range : ImageSubresourceRange{
                aspect_mask : aspect,
                base_array_layer : 0,
                base_mip_level : 0,
                layer_count : 1,
                level_count : 1,
            }
        };
        let view = allocator.device.create_image_view(&image_view_create_info, None).unwrap();
        return Self{
            image,view
        }
    }
    pub unsafe fn destroy(&self, allocator : &mut Allocator){
        allocator.device.destroy_image_view(self.view, None);
        self.image.destroy(allocator);
    }
    pub unsafe fn new_depth(logger : &Logger, allocator : &mut Allocator, extent : Extent2D, format : Format) -> Self{
        return Self::new_2d(logger, allocator, extent, format, MemoryPropertyFlags::DEVICE_LOCAL, ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT, ImageAspectFlags::DEPTH);
    }
    pub unsafe fn new_font(logger : &Logger, allocator : &mut Allocator) -> Self{
        todo!();
    }
}
pub unsafe fn create_swapchain_image_views(device : &Device, format : Format, images : &Vec<Image>) -> Vec<ImageView>{
    let mut views = vec!();
    for &image in images.iter(){
        let image_view_create_info = ImageViewCreateInfo{
            s_type : StructureType::IMAGE_VIEW_CREATE_INFO,
            p_next : std::ptr::null(),
            flags : ImageViewCreateFlags::empty(),
            components : ComponentMapping{r : ComponentSwizzle::R, g : ComponentSwizzle::G, b : ComponentSwizzle::B, a : ComponentSwizzle::A},
            format,
            image,
            view_type : ImageViewType::TYPE_2D,
            subresource_range : ImageSubresourceRange{
                aspect_mask : ImageAspectFlags::COLOR,
                base_array_layer : 0,
                base_mip_level : 0,
                layer_count : 1,
                level_count : 1,
            }
        };
        views.push(device.create_image_view(&image_view_create_info, None).unwrap());
    }
    return views;
}