use ash::extensions::khr::{Surface, Swapchain};
use ash::Instance;
use ash::vk::{ColorSpaceKHR, CompositeAlphaFlagsKHR, Extent2D, Format, FormatFeatureFlags, ImageUsageFlags, PhysicalDevice, PresentModeKHR, SharingMode, StructureType, SurfaceFormatKHR, SurfaceKHR, SurfaceTransformFlagsKHR, SwapchainCreateFlagsKHR, SwapchainCreateInfoKHR, SwapchainKHR};
use slog::{error, info, Logger, warn};
use winit::dpi::PhysicalSize;
use crate::QueueInfo;

const DEPTH_FORMATS : [Format;5] = [Format::D16_UNORM, Format::D16_UNORM_S8_UINT, Format::D24_UNORM_S8_UINT, Format::D32_SFLOAT, Format::D32_SFLOAT_S8_UINT];
const SURFACE_FORMATS : [SurfaceFormatKHR;2] = [SurfaceFormatKHR{format:Format::R8G8B8_SRGB, color_space:ColorSpaceKHR::SRGB_NONLINEAR}, SurfaceFormatKHR{format:Format::B8G8R8_SRGB, color_space:ColorSpaceKHR::SRGB_NONLINEAR}];

pub struct SwapchainInfo{
    pub extent : Extent2D,
    pub format : Format,
    pub depth_format : Format,
    pub color_space : ColorSpaceKHR,
    pub min_image_count : u32,
    pub transform : SurfaceTransformFlagsKHR,
    pub present_mode : PresentModeKHR,
}
impl SwapchainInfo{
    pub unsafe fn new(logger : &Logger, instance : &Instance, physical_device : PhysicalDevice, surface_loader : &Surface, surface : SurfaceKHR, window_size : PhysicalSize<u32>) -> Self{
        let supported_surface_formats = match surface_loader.get_physical_device_surface_formats(physical_device, surface){
            Ok(formats) => {formats}
            Err(error) => {
                error!(logger, "Failed to get surface formats, {}", error);
                panic!();
            }
        };
        let supported_present_modes = match surface_loader.get_physical_device_surface_present_modes(physical_device, surface){
            Ok(present_modes) => {present_modes}
            Err(error) => {
                error!(logger, "Failed to get surface present modes, {}", error);
                panic!();
            }
        };
        let surface_capabilities = match surface_loader.get_physical_device_surface_capabilities(physical_device, surface){
            Ok(capabilities) => {capabilities}
            Err(error) => {
                error!(logger, "Failed to get surface capabilities, {}", error);
                panic!();
            }
        };
        let mut surface_format = None;
        let mut depth_format = None;
        for format in SURFACE_FORMATS{
            if supported_surface_formats.contains(&format){surface_format = Some(format)}
        }
        for format in DEPTH_FORMATS{
            if instance.get_physical_device_format_properties(physical_device, format).optimal_tiling_features.contains(FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT){
                depth_format = Some(format);
            }
        }
        if surface_format.is_none(){surface_format = Some(supported_surface_formats[0])}
        if depth_format.is_none(){
            error!(logger, "Could not find compatible depth format");
            panic!();
        }
        let surface_format = surface_format.unwrap();
        let depth_format = depth_format.unwrap();
        let format = surface_format.format;
        let color_space = surface_format.color_space;
        let present_mode = if supported_present_modes.contains(&PresentModeKHR::MAILBOX){PresentModeKHR::MAILBOX}else{PresentModeKHR::FIFO};
        let extent = if surface_capabilities.current_extent.width!=u32::MAX{surface_capabilities.current_extent} else {Extent2D{width:window_size.width,height:window_size.height}};
        let min_image_count = if surface_capabilities.max_image_count == 0 || surface_capabilities.min_image_count + 1 <= surface_capabilities.max_image_count{
            surface_capabilities.min_image_count + 1
        } else{
            surface_capabilities.max_image_count
        };
        let transform = surface_capabilities.current_transform;
        info!(logger, "Swapchain info:");
        info!(logger, "    Format : {:?}", format);
        info!(logger, "    Depth format: {:?}", depth_format);
        info!(logger, "    Color space: {:?}", color_space);
        info!(logger, "    Extent width: {}, height: {}", extent.width, extent.height);
        info!(logger, "    Transform: {:?}", transform);
        info!(logger, "    Present mode: {:?}", present_mode);
        info!(logger, "    Min image count: {}", min_image_count);
        return Self{
            format,depth_format,color_space,present_mode,extent,min_image_count,transform,
        }
    }
}
pub unsafe fn create_swapchain(logger : &Logger, loader : &Swapchain, info : &SwapchainInfo, queue_info : &QueueInfo, surface : SurfaceKHR) -> SwapchainKHR{
    let sharing_mode = if queue_info.presentation_queue == queue_info.graphics_queue{SharingMode::EXCLUSIVE}else{warn!(logger, "Using concurrent swapchain");SharingMode::CONCURRENT};
    let queue_families = if queue_info.presentation_queue == queue_info.graphics_queue{vec!()}else{vec!(queue_info.graphics_queue,queue_info.presentation_queue)};
    let swapchain_create_info = SwapchainCreateInfoKHR{
        s_type : StructureType::SWAPCHAIN_CREATE_INFO_KHR,
        p_next : std::ptr::null(),
        flags : SwapchainCreateFlagsKHR::empty(),
        surface,
        min_image_count : info.min_image_count,
        present_mode : info.present_mode,
        image_extent : info.extent,
        image_format : info.format,
        image_color_space : info.color_space,
        pre_transform : info.transform,
        image_array_layers : 1,
        clipped  : 1,
        composite_alpha : CompositeAlphaFlagsKHR::OPAQUE,
        old_swapchain : SwapchainKHR::null(),
        image_usage : ImageUsageFlags::COLOR_ATTACHMENT,
        queue_family_index_count : queue_families.len() as u32,
        p_queue_family_indices : queue_families.as_ptr(),
        image_sharing_mode : sharing_mode,
    };
    return match loader.create_swapchain(&swapchain_create_info, None){
        Ok(swapchain) => {
            info!(logger, "    Successfully created swapchain");
            swapchain
        }
        Err(error) => {
            error!(logger, "Failed to create swapchain, {}", error);
            panic!();
        }
    }
}