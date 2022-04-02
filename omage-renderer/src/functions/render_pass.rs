use ash::Device;
use ash::vk::{AccessFlags, AttachmentDescription, AttachmentDescriptionFlags, AttachmentLoadOp, AttachmentReference, AttachmentStoreOp, DependencyFlags, Format, ImageLayout, PipelineBindPoint, PipelineStageFlags, RenderPass, RenderPassCreateFlags, RenderPassCreateInfo, SampleCountFlags, StructureType, SubpassDependency, SubpassDescription, SubpassDescriptionFlags};
use slog::{crit, Logger};

pub unsafe fn create_render_pass(logger : &Logger, device : &Device, format : Format, depth_format : Format) -> RenderPass{
    let attachments = [
        //Swapchain image
        AttachmentDescription{
            flags : AttachmentDescriptionFlags::empty(),
            format,
            initial_layout : ImageLayout::UNDEFINED,
            final_layout : ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            load_op : AttachmentLoadOp::CLEAR,
            store_op : AttachmentStoreOp::STORE,
            stencil_load_op : AttachmentLoadOp::DONT_CARE,
            stencil_store_op : AttachmentStoreOp::DONT_CARE,
            samples : SampleCountFlags::TYPE_1,
        },
        //Depth image
        AttachmentDescription{
            flags : AttachmentDescriptionFlags::empty(),
            format : depth_format,
            initial_layout : ImageLayout::UNDEFINED,
            final_layout : ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
            load_op : AttachmentLoadOp::CLEAR,
            store_op : AttachmentStoreOp::STORE,
            stencil_load_op : AttachmentLoadOp::DONT_CARE,
            stencil_store_op : AttachmentStoreOp::DONT_CARE,
            samples : SampleCountFlags::TYPE_1,
        },
    ];
    let color_attachment_reference = AttachmentReference{
        layout : ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        attachment : 0,
    };
    let depth_attachment_reference = AttachmentReference{
        layout : ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        attachment : 1,
    };
    let subpasses = [
        SubpassDescription{
            flags : SubpassDescriptionFlags::empty(),
            color_attachment_count : 1,
            p_color_attachments : &color_attachment_reference,
            input_attachment_count : 0,
            p_input_attachments : std::ptr::null(),
            p_depth_stencil_attachment : &depth_attachment_reference,
            preserve_attachment_count : 0,
            p_preserve_attachments : std::ptr::null(),
            p_resolve_attachments : std::ptr::null(),
            pipeline_bind_point : PipelineBindPoint::GRAPHICS,
        }
    ];
    let subpass_dependencies = [
        SubpassDependency{
            dependency_flags : DependencyFlags::BY_REGION,
            src_access_mask : AccessFlags::empty(),
            dst_access_mask : AccessFlags::COLOR_ATTACHMENT_WRITE | AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
            src_stage_mask : PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            dst_stage_mask : PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            src_subpass : ash::vk::SUBPASS_EXTERNAL,
            dst_subpass : 0,
        }
    ];
    let render_pass_create_info = RenderPassCreateInfo{
        s_type : StructureType::RENDER_PASS_CREATE_INFO,
        p_next : std::ptr::null(),
        flags : RenderPassCreateFlags::empty(),
        attachment_count : attachments.len() as u32,
        p_attachments : attachments.as_ptr(),
        subpass_count : subpasses.len() as u32,
        p_subpasses : subpasses.as_ptr(),
        dependency_count : subpass_dependencies.len() as u32,
        p_dependencies : subpass_dependencies.as_ptr(),

    };
    return match device.create_render_pass(&render_pass_create_info, None){
        Ok(render_pass) => {render_pass}
        Err(error) => {
            crit!(logger, "[thread#{}]Failed to create render pass, {}.", rayon::current_thread_index().unwrap(), error);
            panic!();
        }
    }
}