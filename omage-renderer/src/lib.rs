use ash::{Device, Entry, Instance};
use ash::extensions::khr::{Surface, Swapchain};
use ash::vk::{SurfaceKHR, SwapchainKHR, ImageView, RenderPass, Framebuffer};
use crossbeam_channel::{Receiver, Sender};
use objects::image::AllocatedImageView;
use slog::{info, Logger};
use omage_util::{FileType, PathManager};
use crate::allocator::Allocator;
use crate::functions::swapchain::SwapchainInfo;
use crate::instance::{RenderConfig, RenderInstance};

pub mod instance;
mod functions;
mod allocator;
pub mod objects;


pub struct Renderer{
    sender : Sender<RenderTask>,
    receiver : Receiver<RenderResult>,
}
impl Renderer{
    pub fn new(instance : RenderInstance, path_manager : PathManager) -> Self{
        let (sender, thread_receiver) = crossbeam_channel::bounded(2);
        let (thread_sender, receiver) = crossbeam_channel::bounded(2);
        rayon::spawn(|| unsafe {
            let renderer = RenderThread::new(instance, path_manager, thread_sender, thread_receiver);
            renderer.listen();
        });
        return Self{
            sender, receiver,
        }
    }
    pub fn stop(&self){
        self.sender.send(RenderTask::Stop).unwrap();
        while self.receiver.recv().unwrap()!=RenderResult::Stopped{};
    }
}

pub struct RenderThread{
    sender : Sender<RenderResult>,
    receiver : Receiver<RenderTask>,
    logger : Logger,
    _entry : Entry,
    config : RenderConfig,
    path_manager : PathManager,
    instance : Instance,
    surface_loader : Surface,
    surface : SurfaceKHR,
    device : Device,
    swapchain_loader : Swapchain,
    swapchain : SwapchainKHR,
    allocator : Allocator,
    swapchain_image_views : Vec<ImageView>,
    depth_image : AllocatedImageView,
    render_pass : RenderPass,
    framebuffers : Vec<Framebuffer>,
}
impl RenderThread{
    pub unsafe fn new(instance : RenderInstance, path_manager : PathManager, sender : Sender<RenderResult>, receiver : Receiver<RenderTask>) -> Self{
        let logger = instance.logger;
        let config = instance.config;
        let surface = instance.surface;
        let entry = instance.entry;
        let instance = instance.instance;
        let surface_loader = Surface::new(&entry, &instance);
        let physical_devices = functions::device::get_compatible_devices(&logger, &instance, &surface_loader, surface);
        let physical_device = functions::device::select_physical_device(&logger, &instance, physical_devices, &config);
        let queue_info = functions::device::QueueInfo::new(&logger, &instance, physical_device);
        let device = functions::device::create_device(&logger, &instance, &queue_info, physical_device, false, false);
        let swapchain_loader = Swapchain::new(&instance, &device);
        let swapchain_info = SwapchainInfo::new(&logger, &instance, &surface_loader, surface, physical_device, false);
        let swapchain = functions::swapchain::create_swapchain(&logger, &swapchain_loader, &swapchain_info, surface);
        let swapchain_images = swapchain_loader.get_swapchain_images(swapchain).unwrap();
        let swapchain_image_views = objects::image::create_swapchain_image_views(&device, swapchain_info.format, &swapchain_images);
        let mut allocator = Allocator::new(&logger, &instance, physical_device, &device);
        let depth_image = objects::image::AllocatedImageView::new_depth(&logger, &mut allocator, swapchain_info.extent, swapchain_info.depth_format);
        let render_pass = functions::render_pass::create_render_pass(&logger, &device, swapchain_info.format, swapchain_info.depth_format);
        let framebuffers = functions::framebuffer::create_framebuffers(&logger, &device, render_pass, &swapchain_image_views, depth_image.view, swapchain_info.extent);
        info!(logger, "[thread#{}]Successfully created the main renderer.", rayon::current_thread_index().unwrap());
        return Self{
            logger,config,surface,surface_loader,_entry:entry,instance,path_manager,sender,receiver,device,swapchain_loader,swapchain,allocator,swapchain_image_views,depth_image,render_pass,framebuffers,
        }
    }
    pub unsafe fn listen(mut self){
        let mut task = self.receiver.try_recv();
        while task != Ok(RenderTask::Stop) {
            if task.is_ok() {
                //let task = task.unwrap();
            }
            self.draw();
            task = self.receiver.try_recv();
        }
        drop(self);
    }
    pub fn draw(&mut self){

    }
    unsafe fn destroy_swapchain(&mut self){
        for &framebuffer in self.framebuffers.iter(){
            self.device.destroy_framebuffer(framebuffer, None);
        }
        self.depth_image.destroy(&mut self.allocator);
        for &swapchain_image_view in self.swapchain_image_views.iter(){
            self.device.destroy_image_view(swapchain_image_view, None);
        }
        self.swapchain_image_views=vec!();
        self.swapchain_loader.destroy_swapchain(self.swapchain, None);
    }
}
impl Drop for RenderThread{
    fn drop(&mut self) {
        self.path_manager.save_file("render", FileType::Config, &self.config);
        unsafe{self.device.device_wait_idle()}.unwrap();
        info!(self.logger, "[thread#{}]Destroying the renderer.",rayon::current_thread_index().unwrap());

        unsafe {
            self.destroy_swapchain();
            self.allocator.destroy();
            self.device.destroy_render_pass(self.render_pass, None);
            self.device.destroy_device(None);
            self.surface_loader.destroy_surface(self.surface, None);
            self.instance.destroy_instance(None);
        }
        self.sender.send(RenderResult::Stopped).unwrap();
    }
}
#[derive(Copy, Clone, PartialEq)]
pub enum RenderTask{
    Stop,
}
#[derive(Copy, Clone, PartialEq)]
pub enum RenderResult{
    Success,
    Stopped,
}