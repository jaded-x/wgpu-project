use wgpu::util::DeviceExt;

use std::f32::consts::FRAC_PI_2;

use cg::{InnerSpace, SquareMatrix};
use instant::Duration;
use winit::{event::{MouseScrollDelta, VirtualKeyCode, ElementState}, dpi::PhysicalPosition};

use crate::util::cast_slice;

use super::input::{InputState, Key};

pub const OPENGL_TO_WGPU_MATRIX: cg::Matrix4<f32> = cg::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;


pub struct Camera {
    pub position: cg::Point3<f32>,
    yaw: cg::Rad<f32>,
    pitch: cg::Rad<f32>,
    pub projection: Projection,
    pub uniform: CameraUniform,
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

impl Camera {
    pub fn new<
        V: Into<cg::Point3<f32>>,
        Y: Into<cg::Rad<f32>>,
        P: Into<cg::Rad<f32>>,
    >(device: &wgpu::Device, camera_bind_group_layout: &wgpu::BindGroupLayout, position: V, yaw: Y, pitch: P, projection: Projection) -> Self {
        let camera_uniform = CameraUniform::new();

        let camera_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: cast_slice(&[camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding()
                }
            ],
            label: Some("camera_bind_group")
        });

        Self {
            position: position.into(),
            yaw: yaw.into(),
            pitch: pitch.into(),
            projection,
            uniform: camera_uniform,
            buffer: camera_buffer,
            bind_group: camera_bind_group,
        }
    }

    pub fn update_uniform(&mut self) {
        let view_proj = (self.projection.calc_matrix() * self.calc_matrix()).into();
        self.uniform.update(&self.position, view_proj);
    }

    pub fn calc_matrix(&self) -> cg::Matrix4<f32> {
        let (sin_pitch, cos_pitch) = self.pitch.0.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.0.sin_cos();

        cg::Matrix4::look_to_rh(
            self.position,
            cg::Vector3::new(
                cos_pitch * cos_yaw,
                sin_pitch,
                cos_pitch * sin_yaw
            ).normalize(),
            cg::Vector3::unit_y(),
        )
    }
}

pub struct Projection {
    aspect: f32,
    fovy: cg::Rad<f32>,
    znear: f32,
    zfar: f32,
}

impl Projection {
    pub fn new<F: Into<cg::Rad<f32>>>(
        width: u32,
        height: u32,
        fovy: F,
        znear: f32,
        zfar: f32,
    ) -> Self {
        Self {
            aspect: width as f32 / height as f32,
            fovy: fovy.into(),
            znear,
            zfar,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    pub fn calc_matrix(&self) -> cg::Matrix4<f32> {
        OPENGL_TO_WGPU_MATRIX * cg::perspective(self.fovy, self.aspect, self.znear, self.zfar)
    }
}


#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct CameraUniform {
    pub view_position: cg::Vector4<f32>,
    pub view_proj: cg::Matrix4<f32>,
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_position: cg::Vector4::new(0.0, 0.0, 0.0, 0.0),
            view_proj: cg::Matrix4::identity(),
        }
    }

    pub fn update(&mut self, eye: &cg::Point3<f32>, view_proj: cg::Matrix4<f32>) {
        self.view_position = eye.to_homogeneous();
        self.view_proj = view_proj;
    }
}

#[derive(Debug)]
pub struct CameraController {
    amount_left: f32,
    amount_right: f32,
    amount_forward: f32,
    amount_backward: f32,
    amount_up: f32,
    amount_down: f32,
    rotate_horizontal: f32,
    rotate_vertical: f32,
    scroll: f32,
    speed: f32,
    sensitivity: f32,
}

impl CameraController {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            amount_left: 0.0,
            amount_right: 0.0,
            amount_forward: 0.0,
            amount_backward: 0.0,
            amount_up: 0.0,
            amount_down: 0.0,
            rotate_horizontal: 0.0,
            rotate_vertical: 0.0,
            scroll: 0.0,
            speed,
            sensitivity,
        }
    }

    pub fn process_keyboard(&mut self, key: VirtualKeyCode, state: ElementState) -> bool{
        let amount = if state == ElementState::Pressed { 1.0 } else { 0.0 };
        match key {
            VirtualKeyCode::W | VirtualKeyCode::Up => {
                self.amount_forward = amount;
                true
            }
            VirtualKeyCode::S | VirtualKeyCode::Down => {
                self.amount_backward = amount;
                true
            }
            VirtualKeyCode::A | VirtualKeyCode::Left => {
                self.amount_left = amount;
                true
            }
            VirtualKeyCode::D | VirtualKeyCode::Right => {
                self.amount_right = amount;
                true
            }
            VirtualKeyCode::Space => {
                self.amount_up = amount;
                true
            }
            VirtualKeyCode::LShift => {
                self.amount_down = amount;
                true
            }
            _ => false,
        }
    }

    pub fn movement(&mut self, input: &InputState) {
        if input.key_down(Key::W) { self.amount_forward = 1.0 } else { self.amount_forward = 0.0 }
        if input.key_down(Key::S) { self.amount_backward = 1.0 } else { self.amount_backward = 0.0 }
        if input.key_down(Key::A) { self.amount_left = 1.0 } else { self.amount_left = 0.0 }
        if input.key_down(Key::D) { self.amount_right = 1.0 } else { self.amount_right = 0.0 }
    }

    pub fn process_mouse(&mut self, mouse_dx: f64, mouse_dy: f64) {
        self.rotate_horizontal = mouse_dx as f32;
        self.rotate_vertical = mouse_dy as f32;
    }

    pub fn process_scroll(&mut self, delta: &MouseScrollDelta) {
        self.scroll = -match delta {
            MouseScrollDelta::LineDelta(_, scroll) => scroll * 100.0,
            MouseScrollDelta::PixelDelta(PhysicalPosition {
                y: scroll,
                ..
            }) => *scroll as f32,
        };
    }

    pub fn update_camera(&mut self, camera: &mut Camera, dt: Duration) {
        let dt = dt.as_secs_f32();

        let (yaw_sin, yaw_cos) = camera.yaw.0.sin_cos();
        let forward = cg::Vector3::new(yaw_cos, 0.0, yaw_sin).normalize();
        let right = cg::Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();
        camera.position += forward * (self.amount_forward - self.amount_backward) * self.speed * dt;
        camera.position += right * (self.amount_right - self.amount_left) * self.speed * dt;

        let (pitch_sin, pitch_cos) = camera.pitch.0.sin_cos();
        let scrollward = cg::Vector3::new(pitch_cos * yaw_cos, pitch_sin, pitch_cos * yaw_sin).normalize();
        camera.position += scrollward * self.scroll * self.speed * self.sensitivity * dt;
        self.scroll = 0.0;

        camera.position.y += (self.amount_up - self.amount_down) * self.speed * dt;

        camera.yaw += cg::Rad(self.rotate_horizontal) * self.sensitivity * dt;
        camera.pitch += cg::Rad(-self.rotate_vertical) * self.sensitivity * dt;

        self.rotate_horizontal = 0.0;
        self.rotate_vertical = 0.0;

        if camera.pitch < -cg::Rad(SAFE_FRAC_PI_2) {
            camera.pitch = -cg::Rad(SAFE_FRAC_PI_2);
        } else if camera.pitch > cg::Rad(SAFE_FRAC_PI_2) {
            camera.pitch = cg::Rad(SAFE_FRAC_PI_2);
        }
    }
}