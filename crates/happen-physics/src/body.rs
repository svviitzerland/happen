use happen_math::Vec3;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub enum BodyType {
    Dynamic,
    Kinematic,
    Static,
}

#[derive(Clone, Debug)]
pub struct RigidBody {
    pub body_type: BodyType,
    pub mass: f32,
    pub inverse_mass: f32,
    pub linear_velocity: Vec3,
    pub angular_velocity: Vec3,
    pub force_accumulator: Vec3,
    pub torque_accumulator: Vec3,
    pub linear_damping: f32,
    pub angular_damping: f32,
    pub restitution: f32,
    pub friction: f32,
    pub gravity_scale: f32,
}

impl RigidBody {
    pub fn dynamic(mass: f32) -> Self {
        Self {
            body_type: BodyType::Dynamic,
            mass,
            inverse_mass: if mass > 0.0 { 1.0 / mass } else { 0.0 },
            linear_velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            force_accumulator: Vec3::ZERO,
            torque_accumulator: Vec3::ZERO,
            linear_damping: 0.01,
            angular_damping: 0.01,
            restitution: 0.3,
            friction: 0.5,
            gravity_scale: 1.0,
        }
    }

    pub fn kinematic() -> Self {
        Self {
            body_type: BodyType::Kinematic,
            mass: 0.0,
            inverse_mass: 0.0,
            linear_velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            force_accumulator: Vec3::ZERO,
            torque_accumulator: Vec3::ZERO,
            linear_damping: 0.0,
            angular_damping: 0.0,
            restitution: 0.3,
            friction: 0.5,
            gravity_scale: 0.0,
        }
    }

    pub fn statik() -> Self {
        Self {
            body_type: BodyType::Static,
            mass: 0.0,
            inverse_mass: 0.0,
            linear_velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            force_accumulator: Vec3::ZERO,
            torque_accumulator: Vec3::ZERO,
            linear_damping: 0.0,
            angular_damping: 0.0,
            restitution: 0.3,
            friction: 0.5,
            gravity_scale: 0.0,
        }
    }

    pub fn apply_force(&mut self, force: Vec3) {
        self.force_accumulator += force;
    }

    pub fn apply_impulse(&mut self, impulse: Vec3) {
        self.linear_velocity += impulse * self.inverse_mass;
    }

    pub fn apply_torque(&mut self, torque: Vec3) {
        self.torque_accumulator += torque;
    }

    pub fn clear_forces(&mut self) {
        self.force_accumulator = Vec3::ZERO;
        self.torque_accumulator = Vec3::ZERO;
    }

    pub fn is_dynamic(&self) -> bool {
        self.body_type == BodyType::Dynamic
    }

    pub fn kinetic_energy(&self) -> f32 {
        0.5 * self.mass * self.linear_velocity.length_squared()
    }

    pub fn momentum(&self) -> Vec3 {
        self.linear_velocity * self.mass
    }
}

