mod rules;
mod body;
mod collider;
mod collision;
mod solver;
mod systems;

pub use rules::{WorldRules, PhysicsZone, PhysicsContext};
pub use body::{RigidBody, BodyType};
pub use collider::{Collider, ColliderShape};
pub use collision::{ContactPoint, ContactManifold, BroadPhase};
pub use solver::ConstraintSolver;
pub use systems::PhysicsPlugin;

pub struct PhysicsConfig {
    pub fixed_timestep: f64,
    pub velocity_iterations: u32,
    pub position_iterations: u32,
    pub max_velocity: f32,
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            fixed_timestep: 1.0 / 60.0,
            velocity_iterations: 8,
            position_iterations: 3,
            max_velocity: 100.0,
        }
    }
}

