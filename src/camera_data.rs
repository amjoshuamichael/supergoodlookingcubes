use glam::{Vec3, Mat4, Quat};

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Copy, Clone, Default)]
pub struct CameraData {
    pub aspect_ratio: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub _padding: [u32; 1],
    pub position: Vec3,
    pub _padding2: f32,
    pub camera: Mat4,
    pub proj: Mat4,
    pub rot: Mat4,
}

impl CameraData {
    pub fn quat(&self) -> Quat {
        Quat::from_rotation_x(self.pitch) * Quat::from_rotation_y(self.yaw)
    }

    pub fn neg_quat(&self) -> Quat {
        Quat::from_rotation_y(-self.yaw) * Quat::from_rotation_x(-self.pitch)
    }

    pub fn quat_frag(&self) -> Quat {
        Quat::from_rotation_y(-self.yaw) * Quat::from_rotation_x(-self.pitch)
    }
}
