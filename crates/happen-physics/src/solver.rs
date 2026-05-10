use happen_math::Vec3;
use crate::body::RigidBody;
use crate::collision::ContactManifold;

pub struct ConstraintSolver {
    pub velocity_iterations: u32,
    pub position_iterations: u32,
    pub baumgarte: f32,
    pub slop: f32,
}

impl Default for ConstraintSolver {
    fn default() -> Self {
        Self {
            velocity_iterations: 8,
            position_iterations: 3,
            baumgarte: 0.2,
            slop: 0.005,
        }
    }
}

impl ConstraintSolver {
    pub fn solve(
        &self,
        manifolds: &[ContactManifold],
        bodies: &mut [(RigidBody, Vec3)],
        entity_to_index: &std::collections::HashMap<u32, usize>,
        dt: f32,
    ) {
        if manifolds.is_empty() {
            return;
        }

        for _ in 0..self.velocity_iterations {
            for manifold in manifolds {
                let idx_a = match entity_to_index.get(&manifold.entity_a.id()) {
                    Some(&i) => i,
                    None => continue,
                };
                let idx_b = match entity_to_index.get(&manifold.entity_b.id()) {
                    Some(&i) => i,
                    None => continue,
                };

                for contact in &manifold.contacts {
                    let inv_mass_a = bodies[idx_a].0.inverse_mass;
                    let inv_mass_b = bodies[idx_b].0.inverse_mass;
                    let inv_mass_sum = inv_mass_a + inv_mass_b;

                    if inv_mass_sum <= 0.0 {
                        continue;
                    }

                    let vel_a = bodies[idx_a].0.linear_velocity;
                    let vel_b = bodies[idx_b].0.linear_velocity;
                    let relative_vel = vel_b - vel_a;
                    let normal_vel = relative_vel.dot(contact.normal);

                    if normal_vel > 0.0 {
                        continue;
                    }

                    let restitution = bodies[idx_a].0.restitution.min(bodies[idx_b].0.restitution);
                    let bias = self.baumgarte / dt
                        * (contact.penetration - self.slop).max(0.0);

                    let j = (-(1.0 + restitution) * normal_vel + bias) / inv_mass_sum;
                    let impulse = contact.normal * j;

                    bodies[idx_a].0.linear_velocity -= impulse * inv_mass_a;
                    bodies[idx_b].0.linear_velocity += impulse * inv_mass_b;

                    let vel_a = bodies[idx_a].0.linear_velocity;
                    let vel_b = bodies[idx_b].0.linear_velocity;
                    let relative_vel = vel_b - vel_a;
                    let tangent = relative_vel - contact.normal * relative_vel.dot(contact.normal);
                    let tangent_len = tangent.length();

                    if tangent_len > 1e-6 {
                        let tangent_dir = tangent / tangent_len;
                        let friction = (bodies[idx_a].0.friction * bodies[idx_b].0.friction).sqrt();
                        let jt = -tangent_len / inv_mass_sum;
                        let jt = jt.clamp(-j * friction, j * friction);
                        let friction_impulse = tangent_dir * jt;

                        bodies[idx_a].0.linear_velocity -= friction_impulse * inv_mass_a;
                        bodies[idx_b].0.linear_velocity += friction_impulse * inv_mass_b;
                    }
                }
            }
        }

        for _ in 0..self.position_iterations {
            for manifold in manifolds {
                let idx_a = match entity_to_index.get(&manifold.entity_a.id()) {
                    Some(&i) => i,
                    None => continue,
                };
                let idx_b = match entity_to_index.get(&manifold.entity_b.id()) {
                    Some(&i) => i,
                    None => continue,
                };

                for contact in &manifold.contacts {
                    let inv_mass_a = bodies[idx_a].0.inverse_mass;
                    let inv_mass_b = bodies[idx_b].0.inverse_mass;
                    let inv_mass_sum = inv_mass_a + inv_mass_b;

                    if inv_mass_sum <= 0.0 {
                        continue;
                    }

                    let correction_mag =
                        (contact.penetration - self.slop).max(0.0) * self.baumgarte;
                    let correction = contact.normal * correction_mag / inv_mass_sum;

                    bodies[idx_a].1 -= correction * inv_mass_a;
                    bodies[idx_b].1 += correction * inv_mass_b;
                }
            }
        }
    }
}
