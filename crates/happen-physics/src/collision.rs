use happen_core::Entity;
use happen_math::{Aabb, Transform, Vec3};
use std::collections::HashMap;

use crate::collider::{Collider, ColliderShape};

#[derive(Clone, Debug)]
pub struct ContactPoint {
    pub point_on_a: Vec3,
    pub point_on_b: Vec3,
    pub normal: Vec3,
    pub penetration: f32,
}

#[derive(Clone, Debug)]
pub struct ContactManifold {
    pub entity_a: Entity,
    pub entity_b: Entity,
    pub contacts: Vec<ContactPoint>,
}

pub struct BroadPhase {
    cell_size: f32,
    cells: HashMap<(i32, i32, i32), Vec<Entity>>,
}

impl BroadPhase {
    pub fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            cells: HashMap::new(),
        }
    }

    fn cell_coord(&self, pos: Vec3) -> (i32, i32, i32) {
        (
            (pos.x / self.cell_size).floor() as i32,
            (pos.y / self.cell_size).floor() as i32,
            (pos.z / self.cell_size).floor() as i32,
        )
    }

    pub fn update(&mut self, entities: &[(Entity, Aabb)]) {
        self.cells.clear();
        for &(entity, ref aabb) in entities {
            let min_cell = self.cell_coord(aabb.min);
            let max_cell = self.cell_coord(aabb.max);

            for x in min_cell.0..=max_cell.0 {
                for y in min_cell.1..=max_cell.1 {
                    for z in min_cell.2..=max_cell.2 {
                        self.cells.entry((x, y, z)).or_default().push(entity);
                    }
                }
            }
        }
    }

    pub fn potential_pairs(&self) -> Vec<(Entity, Entity)> {
        let mut pairs = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for entities in self.cells.values() {
            for i in 0..entities.len() {
                for j in (i + 1)..entities.len() {
                    let a = entities[i];
                    let b = entities[j];
                    let key = if a.id() < b.id() { (a, b) } else { (b, a) };
                    if seen.insert((key.0.id(), key.1.id())) {
                        pairs.push(key);
                    }
                }
            }
        }
        pairs
    }
}

impl Default for BroadPhase {
    fn default() -> Self {
        Self::new(10.0)
    }
}

pub fn narrow_phase(
    entity_a: Entity,
    collider_a: &Collider,
    transform_a: &Transform,
    entity_b: Entity,
    collider_b: &Collider,
    transform_b: &Transform,
) -> Option<ContactManifold> {
    let pos_a = transform_a.position + transform_a.rotation * collider_a.offset;
    let pos_b = transform_b.position + transform_b.rotation * collider_b.offset;

    match (&collider_a.shape, &collider_b.shape) {
        (ColliderShape::Sphere { radius: ra }, ColliderShape::Sphere { radius: rb }) => {
            sphere_vs_sphere(entity_a, pos_a, *ra, entity_b, pos_b, *rb)
        }
        (
            ColliderShape::Sphere { radius },
            ColliderShape::Box { half_extents },
        ) => sphere_vs_box(entity_a, pos_a, *radius, entity_b, pos_b, *half_extents, transform_b),
        (
            ColliderShape::Box { half_extents },
            ColliderShape::Sphere { radius },
        ) => {
            sphere_vs_box(entity_b, pos_b, *radius, entity_a, pos_a, *half_extents, transform_a)
                .map(|mut m| {
                    std::mem::swap(&mut m.entity_a, &mut m.entity_b);
                    for c in &mut m.contacts {
                        std::mem::swap(&mut c.point_on_a, &mut c.point_on_b);
                        c.normal = -c.normal;
                    }
                    m
                })
        }
        (
            ColliderShape::Box { half_extents: he_a },
            ColliderShape::Box { half_extents: he_b },
        ) => box_vs_box(entity_a, pos_a, *he_a, transform_a, entity_b, pos_b, *he_b, transform_b),
        _ => None,
    }
}

fn sphere_vs_sphere(
    ea: Entity, pa: Vec3, ra: f32,
    eb: Entity, pb: Vec3, rb: f32,
) -> Option<ContactManifold> {
    let diff = pb - pa;
    let dist_sq = diff.length_squared();
    let sum_r = ra + rb;

    if dist_sq >= sum_r * sum_r {
        return None;
    }

    let dist = dist_sq.sqrt();
    let normal = if dist > 1e-6 { diff / dist } else { Vec3::Y };
    let penetration = sum_r - dist;

    Some(ContactManifold {
        entity_a: ea,
        entity_b: eb,
        contacts: vec![ContactPoint {
            point_on_a: pa + normal * ra,
            point_on_b: pb - normal * rb,
            normal,
            penetration,
        }],
    })
}

fn sphere_vs_box(
    sphere_entity: Entity, sphere_pos: Vec3, radius: f32,
    box_entity: Entity, box_pos: Vec3, half_extents: Vec3,
    box_transform: &Transform,
) -> Option<ContactManifold> {
    let local_sphere = box_transform.rotation.inverse() * (sphere_pos - box_pos);

    let clamped = Vec3::new(
        local_sphere.x.clamp(-half_extents.x, half_extents.x),
        local_sphere.y.clamp(-half_extents.y, half_extents.y),
        local_sphere.z.clamp(-half_extents.z, half_extents.z),
    );

    let diff = local_sphere - clamped;
    let dist_sq = diff.length_squared();

    if dist_sq >= radius * radius {
        return None;
    }

    let dist = dist_sq.sqrt();
    let local_normal = if dist > 1e-6 {
        diff / dist
    } else {
        let abs_local = Vec3::new(
            half_extents.x - local_sphere.x.abs(),
            half_extents.y - local_sphere.y.abs(),
            half_extents.z - local_sphere.z.abs(),
        );
        if abs_local.x < abs_local.y && abs_local.x < abs_local.z {
            Vec3::new(local_sphere.x.signum(), 0.0, 0.0)
        } else if abs_local.y < abs_local.z {
            Vec3::new(0.0, local_sphere.y.signum(), 0.0)
        } else {
            Vec3::new(0.0, 0.0, local_sphere.z.signum())
        }
    };

    let world_normal = box_transform.rotation * local_normal;
    let world_closest = box_pos + box_transform.rotation * clamped;

    Some(ContactManifold {
        entity_a: sphere_entity,
        entity_b: box_entity,
        contacts: vec![ContactPoint {
            point_on_a: sphere_pos - world_normal * radius,
            point_on_b: world_closest,
            normal: world_normal,
            penetration: radius - dist,
        }],
    })
}

fn box_vs_box(
    ea: Entity, pa: Vec3, he_a: Vec3, ta: &Transform,
    eb: Entity, pb: Vec3, he_b: Vec3, tb: &Transform,
) -> Option<ContactManifold> {
    let diff = pb - pa;
    let overlap_x = (he_a.x + he_b.x) - diff.x.abs();
    let overlap_y = (he_a.y + he_b.y) - diff.y.abs();
    let overlap_z = (he_a.z + he_b.z) - diff.z.abs();

    if overlap_x <= 0.0 || overlap_y <= 0.0 || overlap_z <= 0.0 {
        return None;
    }

    let (normal, penetration) = if overlap_x < overlap_y && overlap_x < overlap_z {
        (Vec3::new(diff.x.signum(), 0.0, 0.0), overlap_x)
    } else if overlap_y < overlap_z {
        (Vec3::new(0.0, diff.y.signum(), 0.0), overlap_y)
    } else {
        (Vec3::new(0.0, 0.0, diff.z.signum()), overlap_z)
    };

    let _ = (ta, tb);

    Some(ContactManifold {
        entity_a: ea,
        entity_b: eb,
        contacts: vec![ContactPoint {
            point_on_a: pa + normal * Vec3::new(he_a.x, he_a.y, he_a.z),
            point_on_b: pb - normal * Vec3::new(he_b.x, he_b.y, he_b.z),
            normal,
            penetration,
        }],
    })
}
