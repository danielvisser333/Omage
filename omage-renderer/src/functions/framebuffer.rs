use ash::Device;
use ash::vk::{Extent2D, Framebuffer, FramebufferCreateFlags, FramebufferCreateInfo, ImageView, RenderPass, StructureType};
use slog::{crit, Logger};

pub unsafe fn create_framebuffers(logger : &Logger, device : &Device, render_pass : RenderPass, swapchain_images : &Vec<ImageView>, depth_image_view : ImageView, extent : Extent2D) -> Vec<Framebuffer>{
    let mut framebuffers = vec!();
    for &swapchain_image in swapchain_images.iter(){
        let attachments = [swapchain_image, depth_image_view];
        let framebuffer_create_info = FramebufferCreateInfo{
            s_type : StructureType::FRAMEBUFFER_CREATE_INFO,
            p_next : std::ptr::null(),
            flags : FramebufferCreateFlags::empty(),
            attachment_count : attachments.len() as u32,
            p_attachments : attachments.as_ptr(),
            height : extent.height,
            width : extent.width,
            layers : 1,
            render_pass,
        };
        framebuffers.push(match device.create_framebuffer(&framebuffer_create_info, None){
            Ok(framebuffer) => {framebuffer}
            Err(error) => {
                crit!(logger, "[thread#{}]Failed to create framebuffer, {}.", rayon::current_thread_index().unwrap(), error);
                panic!();
            }
        });
    }
    return framebuffers;
}