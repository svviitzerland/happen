use glam::{Mat4, Quat, Vec3};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Default for Transform {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Transform {
    pub const IDENTITY: Self = Self {
        position: Vec3::ZERO,
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    };

    pub fn from_position(position: Vec3) -> Self {
        Self {
            position,
            ..Self::IDENTITY
        }
    }

    pub fn from_position_rotation(position: Vec3, rotation: Quat) -> Self {
        Self {
            position,
            rotation,
            ..Self::IDENTITY
        }
    }

    pub fn to_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position)
    }

    pub fn normal_matrix(&self) -> Mat4 {
        let inv_scale = Vec3::new(1.0 / self.scale.x, 1.0 / self.scale.y, 1.0 / self.scale.z);
        Mat4::from_scale_rotation_translation(inv_scale, self.rotation, Vec3::ZERO)
    }

    pub fn forward(&self) -> Vec3 {
        self.rotation * -Vec3::Z
    }

    pub fn right(&self) -> Vec3 {
        self.rotation * Vec3::X
    }

    pub fn up(&self) -> Vec3 {
        self.rotation * Vec3::Y
    }

    pub fn lerp(&self, other: &Transform, t: f32) -> Transform {
        Transform {
            position: self.position.lerp(other.position, t),
            rotation: self.rotation.slerp(other.rotation, t),
            scale: self.scale.lerp(other.scale, t),
        }
    }

    pub fn inverse(&self) -> Transform {
        let inv_rotation = self.rotation.inverse();
        let inv_scale = Vec3::new(1.0 / self.scale.x, 1.0 / self.scale.y, 1.0 / self.scale.z);
        let inv_position = inv_rotation * (-self.position * inv_scale);
        Transform {
            position: inv_position,
            rotation: inv_rotation,
            scale: inv_scale,
        }
    }

    pub fn compose(parent: &Transform, child: &Transform) -> Transform {
        Transform {
            position: parent.position + parent.rotation * (parent.scale * child.position),
            rotation: parent.rotation * child.rotation,
            scale: parent.scale * child.scale,
        }
    }

    pub fn transform_point(&self, point: Vec3) -> Vec3 {
        self.position + self.rotation * (self.scale * point)
    }

    pub fn transform_direction(&self, dir: Vec3) -> Vec3 {
        self.rotation * dir
    }
}
