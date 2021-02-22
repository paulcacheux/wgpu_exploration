use futures::executor::block_on;
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

mod camera;
mod instance;
mod state;
mod texture;
mod vertex;

use state::State;

fn main() {
    env_logger::init();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("WGPU Learning")
        .build(&event_loop)
        .expect("Failed to create window");

    window.set_cursor_grab(true).expect("Failed to grab cursor");
    window.set_cursor_visible(false);

    let mut state = block_on(State::new(&window));
    let mut last_time = std::time::Instant::now();

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent { event, window_id } if window_id == window.id() => {
            if !state.handle_window_event(&event) {
                match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::KeyboardInput { input, .. } => match input {
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        } => *control_flow = ControlFlow::Exit,
                        _ => {}
                    },
                    WindowEvent::Resized(new_size) => {
                        state.resize(new_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        state.resize(*new_inner_size);
                    }
                    _ => {}
                }
            }
        }
        Event::DeviceEvent { event, .. } => {
            state.handle_device_event(&event);
        }
        Event::RedrawRequested(window_id) if window_id == window.id() => match state.render() {
            Ok(_) => {}
            Err(wgpu::SwapChainError::Lost) => state.resize(state.window_size),
            Err(wgpu::SwapChainError::OutOfMemory) => *control_flow = ControlFlow::Exit,
            Err(e) => eprintln!("{:?}", e),
        },
        Event::MainEventsCleared => {
            let current_time = std::time::Instant::now();
            let elapsed = current_time.duration_since(last_time);
            last_time = current_time;

            state.update(elapsed);
            window.request_redraw();
        }
        _ => {}
    })
}
