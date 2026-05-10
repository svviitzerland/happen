use glam::Vec3;
use crate::Aabb;

#[derive(Debug, Clone, Copy)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

impl Ray {
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
        }
    }

    pub fn point_at(&self, t: f32) -> Vec3 {
        self.origin + self.direction * t
    }

    pub fn intersect_aabb(&self, aabb: &Aabb) -> Option<f32> {
        let inv_dir = Vec3::new(1.0 / self.direction.x, 1.0 / self.direction.y, 1.0 / self.direction.z);

        let t1 = (aabb.min.x - self.origin.x) * inv_dir.x;
        let t2 = (aabb.max.x - self.origin.x) * inv_dir.x;
        let t3 = (aabb.min.y - self.origin.y) * inv_dir.y;
        let t4 = (aabb.max.y - self.origin.y) * inv_dir.y;
        let t5 = (aabb.min.z - self.origin.z) * inv_dir.z;
        let t6 = (aabb.max.z - self.origin.z) * inv_dir.z;

        let tmin = t1.min(t2).max(t3.min(t4)).max(t5.min(t6));
        let tmax = t1.max(t2).min(t3.max(t4)).min(t5.max(t6));

        if tmax < 0.0 || tmin > tmax {
            return None;
        }

        Some(if tmin < 0.0 { tmax } else { tmin })
    }

    pub fn intersect_plane(&self, normal: Vec3, d: f32) -> Option<f32> {
        let denom = normal.dot(self.direction);
        if denom.abs() < 1e-6 {
            return None;
        }
        let t = -(normal.dot(self.origin) + d) / denom;
        if t >= 0.0 { Some(t) } else { None }
    }

    pub fn intersect_sphere(&self, center: Vec3, radius: f32) -> Option<f32> {
        let oc = self.origin - center;
        let a = self.direction.dot(self.direction);
        let b = 2.0 * oc.dot(self.direction);
        let c = oc.dot(oc) - radius * radius;
        let discriminant = b * b - 4.0 * a * c;

        if discriminant < 0.0 {
            return None;
        }

        let sqrt_disc = discriminant.sqrt();
        let t1 = (-b - sqrt_disc) / (2.0 * a);
        let t2 = (-b + sqrt_disc) / (2.0 * a);

        if t1 >= 0.0 {
            Some(t1)
        } else if t2 >= 0.0 {
            Some(t2)
        } else {
            None
        }
    }
}
