use ash::Device;
use ash::vk::{ComponentMapping, ComponentSwizzle, Format, Image, ImageAspectFlags, ImageSubresourceRange, ImageView, ImageViewCreateFlags, ImageViewCreateInfo, ImageViewType, StructureType};
use slog::{error, Logger};

pub unsafe fn create_image_view(logger : &Logger, device : &Device, image : Image, format : Format, view_type : ImageViewType, components : ComponentMapping, subresource_range : ImageSubresourceRange) -> ImageView{
    let image_view_create_info = ImageViewCreateInfo{
        s_type : StructureType::IMAGE_VIEW_CREATE_INFO,
        p_next : std::ptr::null(),
        flags : ImageViewCreateFlags::empty(),
        format,
        image,
        view_type,
        components,
        subresource_range
    };
    return match device.create_image_view(&image_view_create_info, None){
        Ok(view) => {view}
        Err(error) => {
            error!(logger, "Failed to create image view, {}", error);
            panic!();
        }
    }
}
pub unsafe fn create_swapchain_image_views(logger : &Logger, device : &Device, images : &Vec<Image>, format : Format) -> Vec<ImageView>{
    let mut views = vec!();
    let components = ComponentMapping{
        r : ComponentSwizzle::R,
        g : ComponentSwizzle::G,
        b : ComponentSwizzle::B,
        a : ComponentSwizzle::A,
    };
    let subresource_range = ImageSubresourceRange{
        aspect_mask : ImageAspectFlags::COLOR,
        base_array_layer : 0,
        base_mip_level : 0,
        layer_count : 1,
        level_count : 1,
    };
    for &image in images.iter(){
        views.push(create_image_view(logger, device, image, format, ImageViewType::TYPE_2D, components, subresource_range));
    }
    return views;
}
