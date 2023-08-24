use glam::{Vec3, Mat4, EulerRot, Quat};

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Copy, Clone, Default)]
pub struct CameraData {
    pub aspect_ratio: f32,
    pub yaw: f32,
    pub _padding: [u32; 2],
    pub position: Vec3,
    pub _padding2: f32,
    pub camera: Mat4,
    pub proj: Mat4,
    pub rot: Mat4,
}

impl CameraData {
    pub fn quat(&self) -> Quat {
        glam::Quat::from_euler(
            EulerRot::YXZ,
            self.yaw,
            0.0,
            0.0,
        )
    }

    pub fn neg_quat(&self) -> Quat {
        glam::Quat::from_euler(
            EulerRot::YXZ,
            -self.yaw,
            0.0,
            0.0,
        )
    }
}
