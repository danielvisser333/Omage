use ash::extensions::khr::{Surface, Swapchain};
use ash::Instance;
use ash::vk::{ColorSpaceKHR, CompositeAlphaFlagsKHR, Extent2D, Format, FormatFeatureFlags, ImageUsageFlags, PhysicalDevice, PresentModeKHR, SharingMode, StructureType, SurfaceFormatKHR, SurfaceKHR, SurfaceTransformFlagsKHR, SwapchainCreateFlagsKHR, SwapchainCreateInfoKHR, SwapchainKHR};
use slog::{crit, Logger};

pub const FORMATS : [SurfaceFormatKHR;2] = [
    SurfaceFormatKHR{format:Format::R8G8B8A8_SRGB, color_space : ColorSpaceKHR::SRGB_NONLINEAR},
    SurfaceFormatKHR{format:Format::B8G8R8A8_SRGB, color_space : ColorSpaceKHR::SRGB_NONLINEAR},
];
pub const DEPTH_STENCIL_FORMATS : [Format; 3] = [
    Format::D16_UNORM_S8_UINT, Format::D24_UNORM_S8_UINT, Format::D32_SFLOAT_S8_UINT,
];
pub const DEPTH_ONLY_FORMATS : [Format; 2] = [
    Format::D16_UNORM, Format::D32_SFLOAT,
];

pub struct SwapchainInfo{
    pub extent : Extent2D,
    present_mode : PresentModeKHR,
    transform : SurfaceTransformFlagsKHR,
    pub format : Format,
    color_space : ColorSpaceKHR,
    pub depth_format : Format,
    min_image_count : u32,
}
impl SwapchainInfo{
    pub unsafe fn new(logger : &Logger, instance : &Instance, surface_loader : &Surface, surface : SurfaceKHR, device : PhysicalDevice, stencil_buffering : bool) -> Self{
        let supported_formats = surface_loader.get_physical_device_surface_formats(device, surface).unwrap();
        let supported_present_modes = surface_loader.get_physical_device_surface_present_modes(device, surface).unwrap();
        let capabilities = surface_loader.get_physical_device_surface_capabilities(device, surface).unwrap();
        let mut format = None;
        let mut depth_format = None;
        for srgb_format in FORMATS{
            if supported_formats.contains(&srgb_format){format = Some(srgb_format); break;}
        }
        if !stencil_buffering{
            for allowed_format in DEPTH_ONLY_FORMATS{
                if instance.get_physical_device_format_properties(device, allowed_format).optimal_tiling_features.contains(FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT){depth_format = Some(allowed_format)}
            }
        }
        for allowed_format in DEPTH_STENCIL_FORMATS{
            if instance.get_physical_device_format_properties(device, allowed_format).optimal_tiling_features.contains(FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT){depth_format = Some(allowed_format)}
        }
        let format = match format{
            Some(format) => {format}
            None => {supported_formats[0]}
        };
        let depth_format = match depth_format{
            Some(depth_format) => {depth_format}
            None => {crit!(logger, "[thread#{}]Failed to get supported depth format.", rayon::current_thread_index().unwrap());panic!()}
        };
        let min_image_count = if capabilities.min_image_count + 1 <= capabilities.max_image_count || capabilities.max_image_count == 0 {capabilities.min_image_count + 1} else {capabilities.max_image_count};
        return Self{
            extent : if capabilities.current_extent.width!=u32::MAX{capabilities.current_extent}else{Extent2D{width:0,height:0}},
            present_mode : if supported_present_modes.contains(&PresentModeKHR::MAILBOX){PresentModeKHR::MAILBOX}else{PresentModeKHR::FIFO},
            transform : capabilities.current_transform,
            format : format.format,
            color_space : format.color_space,
            depth_format,
            min_image_count,
        }
    }
}
pub unsafe fn create_swapchain(logger : &Logger, loader : &Swapchain, info : &SwapchainInfo, surface : SurfaceKHR) -> SwapchainKHR{
    let swapchain_create_info = SwapchainCreateInfoKHR{
        s_type : StructureType::SWAPCHAIN_CREATE_INFO_KHR,
        p_next : std::ptr::null(),
        flags : SwapchainCreateFlagsKHR::empty(),
        surface,
        min_image_count : info.min_image_count,
        present_mode : info.present_mode,
        image_format : info.format,
        image_color_space : info.color_space,
        pre_transform : info.transform,
        image_extent : info.extent,
        clipped : 1,
        composite_alpha : CompositeAlphaFlagsKHR::OPAQUE,
        image_array_layers : 1,
        image_usage : ImageUsageFlags::COLOR_ATTACHMENT,
        image_sharing_mode : SharingMode::EXCLUSIVE,
        queue_family_index_count : 0,
        p_queue_family_indices : std::ptr::null(),
        old_swapchain : SwapchainKHR::null(),
    };
    return match loader.create_swapchain(&swapchain_create_info, None){
        Ok(swapchain) => {swapchain}
        Err(error) => {
            crit!(logger, "[thread#{}]Failed to create swapchain, {}.", rayon::current_thread_index().unwrap(), error);
            panic!();
        }
    }
}