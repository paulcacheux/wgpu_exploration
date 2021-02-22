use winit::event::{DeviceEvent, ElementState, KeyboardInput, VirtualKeyCode, WindowEvent};

type Point3 = cgmath::Point3<f32>;
type Vec3 = cgmath::Vector3<f32>;

#[derive(Debug, Clone)]
pub struct Camera {
    pub eye: Point3,
    pub front: Vec3,
    pub up: Vec3,

    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

impl Camera {
    pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.eye + self.front, self.up);
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);
        return OPENGL_TO_WGPU_MATRIX * proj * view;
    }
}

pub struct CameraController {
    speed: f32,
    sensitivity: f32,

    is_up_pressed: bool,
    is_down_pressed: bool,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,

    yaw: f32,
    pitch: f32,
}

impl CameraController {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            speed,
            sensitivity,
            is_up_pressed: false,
            is_down_pressed: false,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,

            yaw: -90.0,
            pitch: 0.0,
        }
    }

    pub fn process_window_event(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state,
                        virtual_keycode: Some(keycode),
                        ..
                    },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                match keycode {
                    VirtualKeyCode::Space => {
                        self.is_up_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::LShift => {
                        self.is_down_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::Z | VirtualKeyCode::Up => {
                        self.is_forward_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::Q | VirtualKeyCode::Left => {
                        self.is_left_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::S | VirtualKeyCode::Down => {
                        self.is_backward_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::D | VirtualKeyCode::Right => {
                        self.is_right_pressed = is_pressed;
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    pub fn process_device_event(&mut self, event: &DeviceEvent) -> bool {
        match event {
            DeviceEvent::MouseMotion { delta: (dx, dy) } => {
                let dx = *dx as f32 * self.sensitivity;
                let dy = *dy as f32 * self.sensitivity;

                self.yaw += dx;
                self.pitch -= dy;

                self.pitch = self
                    .pitch
                    .clamp(-std::f32::consts::FRAC_PI_2, std::f32::consts::FRAC_PI_2);

                true
            }
            _ => false,
        }
    }

    pub fn update_camera(&self, camera: &mut Camera, dt: std::time::Duration) {
        use cgmath::InnerSpace;

        let speed = self.speed * dt.as_secs_f32();

        if self.is_forward_pressed {
            camera.eye += speed * camera.front;
        }
        if self.is_backward_pressed {
            camera.eye -= speed * camera.front;
        }
        if self.is_left_pressed {
            camera.eye -= speed * camera.front.cross(camera.up).normalize();
        }
        if self.is_right_pressed {
            camera.eye += speed * camera.front.cross(camera.up).normalize();
        }
        if self.is_up_pressed {
            camera.eye += speed * camera.up;
        }
        if self.is_down_pressed {
            camera.eye -= speed * camera.up;
        }

        let direction = cgmath::Vector3 {
            x: self.yaw.cos() * self.pitch.cos(),
            y: self.pitch.sin(),
            z: self.yaw.sin() * self.pitch.cos(),
        };
        camera.front = direction.normalize();
    }
}
