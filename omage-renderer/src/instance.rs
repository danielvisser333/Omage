use ash::{Entry, Instance};
use ash::vk::SurfaceKHR;
use slog::{crit, Logger};
use winit::window::Window;
use serde_derive::{Serialize, Deserialize};

pub struct RenderInstance{
    pub config : RenderConfig,
    pub logger : Logger,
    pub entry : Entry,
    pub instance : Instance,
    pub surface : SurfaceKHR,
}
impl RenderInstance{
    pub unsafe fn new(logger : Logger, window : &Window, config : RenderConfig) -> Self{
        let entry = match Entry::load(){Ok(entry)=>{entry}Err(error)=>{crit!(logger, "Failed to load Vulkan driver, {}.",error);panic!()}};
        let instance = crate::functions::instance::create_instance(&logger, &entry, config.debugging, window).unwrap();
        let surface = match ash_window::create_surface(&entry, &instance, &window, None){
            Ok(surface) => {surface}
            Err(error) => {crit!(logger, "[thread#{}]Failed to create Vulkan surface, {}.", rayon::current_thread_index().unwrap(), error);panic!()}
        };
        return Self{
            logger,entry,config,instance,surface,
        }
    }
}
#[derive(Serialize, Deserialize)]
pub struct RenderConfig{
    pub debugging : bool,
    pub gpu : String,
}
impl Default for RenderConfig{
    fn default() -> Self {
        return Self{
            debugging : false,
            gpu : String::new(),
        }
    }
}