pub use glam::{Mat3, Mat4, Quat, Vec2, Vec3, Vec4};

mod transform;
mod aabb;
mod ray;
mod color;

pub use transform::Transform;
pub use aabb::Aabb;
pub use ray::Ray;
pub use color::Color;

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

pub fn inverse_lerp(a: f32, b: f32, value: f32) -> f32 {
    if (b - a).abs() < f32::EPSILON {
        0.0
    } else {
        (value - a) / (b - a)
    }
}

pub fn remap(value: f32, from: (f32, f32), to: (f32, f32)) -> f32 {
    let t = inverse_lerp(from.0, from.1, value);
    lerp(to.0, to.1, t)
}

pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
