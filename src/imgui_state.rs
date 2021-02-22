use imgui::{Context, FontSource};
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use winit::{event::Event, window::Window};

pub struct ImguiState {
    pub context: imgui::Context,
    pub platform: WinitPlatform,
}

impl ImguiState {
    pub fn create(window: &Window) -> Self {
        let mut imgui = Context::create();
        imgui.set_ini_filename(None);
        let mut imgui_platform = WinitPlatform::init(&mut imgui);
        imgui_platform.attach_window(imgui.io_mut(), &window, HiDpiMode::Default);

        let hidpi_factor = window.scale_factor();
        let font_size = (13.0 * hidpi_factor) as f32;
        imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;
        imgui.fonts().add_font(&[FontSource::DefaultFontData {
            config: Some(imgui::FontConfig {
                oversample_h: 1,
                pixel_snap_h: true,
                size_pixels: font_size,
                ..Default::default()
            }),
        }]);

        ImguiState {
            context: imgui,
            platform: imgui_platform,
        }
    }

    pub fn handle_event(&mut self, window: &Window, event: &Event<()>) {
        self.platform
            .handle_event(self.context.io_mut(), window, event);
    }

    pub fn prepare_frame(&mut self, elapsed: std::time::Duration, window: &Window) {
        self.context.io_mut().update_delta_time(elapsed);
        self.platform
            .prepare_frame(self.context.io_mut(), window)
            .expect("Failed to prepare imgui frame")
    }
}
