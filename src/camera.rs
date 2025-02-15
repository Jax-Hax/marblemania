use glam::{Mat4, Vec3};
use instant::Duration;
use wgpu::util::DeviceExt;
use wgpu::{Device, SurfaceConfiguration, Buffer, BindGroupLayout, BindGroup};

use crate::app_resource::App;
use crate::structs::CameraController;
use crate::state::State;
use std::f32::consts::FRAC_PI_2;
const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;
#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Mat4 = Mat4::from_cols_array(
    &[1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0]
);



#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_position: [f32; 4],
    pub view_proj: [[f32; 4]; 4],
}
impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_position: [0.0; 4],
            view_proj: Mat4::IDENTITY.to_cols_array_2d(), //maybe here
        }
    }
    pub fn update_screen_size(&mut self, width: u32, height: u32) {
        self.view_position[3] = (width as f32) /(height as f32);
    }
    pub fn update_view_proj(&mut self, camera: &Camera, projection: &Projection) {
        self.view_position = camera.position.extend(self.view_position[3]).into();
        self.view_proj = (projection.calc_matrix() * camera.calc_matrix()).to_cols_array_2d();
    }
}
pub struct CameraStruct{
    pub projection: Projection,
    pub camera_uniform: CameraUniform,
    pub buffer: Buffer,
    pub bind_group_layout: BindGroupLayout,
    pub bind_group: BindGroup,
    pub camera_transform: Camera,
    pub camera_controller: CameraController
}
impl CameraStruct{
    pub fn new(device: &Device, config: &SurfaceConfiguration, camera: Camera, camera_controller: CameraController) -> Self{
        let projection = Projection::new(config.width, config.height, f32::to_radians(45.0), 0.1, 100.0);
    
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera, &projection);
        camera_uniform.update_screen_size(config.width, config.height);

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
    
        let bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });
    
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });
        Self {projection,camera_uniform, buffer, bind_group_layout, bind_group, camera_transform: camera, camera_controller }
    }
    pub fn update_view_proj(&mut self){
        self
            .camera_uniform
            .update_view_proj(&self.camera_transform, &self.projection);
    }
}


#[derive(Debug)]
pub struct Camera {
    pub position: Vec3,
    pub yaw: f32, //angle in radians
    pub pitch: f32, //angle in radians
}

impl Camera {
    pub fn new(
        position: Vec3,
        yaw: f32,
        pitch: f32,
    ) -> Self {
        Self {
            position: position.into(),
            yaw,
            pitch,
        }
    }

    pub fn calc_matrix(&self) -> Mat4 {
        let (sin_pitch, cos_pitch) = self.pitch.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.sin_cos();

        Mat4::look_to_rh(
            self.position,
            Vec3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw).normalize(),
            Vec3::Y,
        )
    }
}

pub struct Projection {
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

impl Projection {
    pub fn new<F: Into<f32>>(width: u32, height: u32, fovy: F, znear: f32, zfar: f32) -> Self {
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

    pub fn calc_matrix(&self) -> Mat4 {
        OPENGL_TO_WGPU_MATRIX * Mat4::perspective_rh_gl(self.fovy, self.aspect, self.znear, self.zfar)
    }
}

pub fn default_3d_cam(state: &mut State, dt: Duration) {
    let dt = dt.as_secs_f32();
    let cam = &mut state.world
                .get_resource_mut::<App>()
                .unwrap().camera;
    // Move forward/backward and left/right
    let (yaw_sin, yaw_cos) = cam.camera_transform.yaw.sin_cos();
    let forward = Vec3::new(yaw_cos, 0.0, yaw_sin).normalize();
    let right = Vec3::new(-yaw_sin, 0.0, yaw_cos).normalize();
    cam.camera_transform.position += forward * (cam.camera_controller.amount_forward - cam.camera_controller.amount_backward) * cam.camera_controller.speed * dt;
    cam.camera_transform.position += right * (cam.camera_controller.amount_right - cam.camera_controller.amount_left) * cam.camera_controller.speed * dt;

    // Move in/out (aka. "zoom")
    // Note: this isn't an actual zoom. The camera's position
    // changes when zooming. I've added this to make it easier
    // to get closer to an object you want to focus on.
    let (pitch_sin, pitch_cos) = cam.camera_transform.pitch.sin_cos();
    let scrollward =
    Vec3::new(pitch_cos * yaw_cos, pitch_sin, pitch_cos * yaw_sin).normalize();
    cam.camera_transform.position += scrollward * cam.camera_controller.scroll * cam.camera_controller.speed * cam.camera_controller.sensitivity * dt;
    cam.camera_controller.scroll = 0.0;

    // Move up/down. Since we don't use roll, we can just
    // modify the y coordinate directly.
    cam.camera_transform.position.y += (cam.camera_controller.amount_up - cam.camera_controller.amount_down) * cam.camera_controller.speed * dt;

    // Rotate
    cam.camera_transform.yaw += cam.camera_controller.rotate_horizontal * cam.camera_controller.sensitivity * dt;
    cam.camera_transform.pitch += -cam.camera_controller.rotate_vertical * cam.camera_controller.sensitivity * dt;

    // If process_mouse isn't called every frame, these values
    // will not get set to zero, and the camera will rotate
    // when moving in a non cardinal direction.
    cam.camera_controller.rotate_horizontal = 0.0;
    cam.camera_controller.rotate_vertical = 0.0;

    // Keep the camera's angle from going too high/low.
    if cam.camera_transform.pitch < -SAFE_FRAC_PI_2 {
        cam.camera_transform.pitch = -SAFE_FRAC_PI_2;
    } else if cam.camera_transform.pitch > SAFE_FRAC_PI_2 {
        cam.camera_transform.pitch = SAFE_FRAC_PI_2;
    }
}
