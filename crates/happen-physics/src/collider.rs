use happen_math::{Aabb, Transform, Vec3};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ColliderShape {
    Sphere { radius: f32 },
    Box { half_extents: Vec3 },
    Capsule { radius: f32, half_height: f32 },
}

impl ColliderShape {
    pub fn compute_aabb(&self, transform: &Transform) -> Aabb {
        match self {
            ColliderShape::Sphere { radius } => Aabb::from_center_half_extents(
                transform.position,
                Vec3::splat(*radius * transform.scale.max_element()),
            ),
            ColliderShape::Box { half_extents } => {
                let scaled = *half_extents * transform.scale;
                let axes = [
                    transform.rotation * Vec3::X * scaled.x,
                    transform.rotation * Vec3::Y * scaled.y,
                    transform.rotation * Vec3::Z * scaled.z,
                ];
                let extent = Vec3::new(
                    axes[0].x.abs() + axes[1].x.abs() + axes[2].x.abs(),
                    axes[0].y.abs() + axes[1].y.abs() + axes[2].y.abs(),
                    axes[0].z.abs() + axes[1].z.abs() + axes[2].z.abs(),
                );
                Aabb::from_center_half_extents(transform.position, extent)
            }
            ColliderShape::Capsule {
                radius,
                half_height,
            } => {
                let r = *radius * transform.scale.max_element();
                let h = *half_height * transform.scale.y;
                Aabb::from_center_half_extents(
                    transform.position,
                    Vec3::new(r, h + r, r),
                )
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Collider {
    pub shape: ColliderShape,
    pub offset: Vec3,
    pub is_trigger: bool,
}

impl Collider {
    pub fn sphere(radius: f32) -> Self {
        Self {
            shape: ColliderShape::Sphere { radius },
            offset: Vec3::ZERO,
            is_trigger: false,
        }
    }

    pub fn cuboid(half_extents: Vec3) -> Self {
        Self {
            shape: ColliderShape::Box { half_extents },
            offset: Vec3::ZERO,
            is_trigger: false,
        }
    }

    pub fn capsule(radius: f32, half_height: f32) -> Self {
        Self {
            shape: ColliderShape::Capsule {
                radius,
                half_height,
            },
            offset: Vec3::ZERO,
            is_trigger: false,
        }
    }

    pub fn as_trigger(mut self) -> Self {
        self.is_trigger = true;
        self
    }

    pub fn with_offset(mut self, offset: Vec3) -> Self {
        self.offset = offset;
        self
    }

    pub fn world_aabb(&self, transform: &Transform) -> Aabb {
        let mut t = *transform;
        t.position += transform.rotation * self.offset;
        self.shape.compute_aabb(&t)
    }
}

