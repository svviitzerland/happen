use happen_core::{App, Plugin, World};
use happen_math::Transform;

use crate::body::{BodyType, RigidBody};
use crate::collider::Collider;
use crate::collision::{narrow_phase, BroadPhase, ContactManifold};
use crate::rules::PhysicsContext;
use crate::solver::ConstraintSolver;
use crate::PhysicsConfig;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PhysicsContext::default());
        app.insert_resource(PhysicsConfig::default());
        app.add_system(
            happen_core::STAGE_UPDATE,
            "physics_step",
            physics_step_system,
        );
    }

    fn name(&self) -> &str {
        "PhysicsPlugin"
    }
}

fn physics_step_system(world: &mut World) {
    let dt = world
        .get_resource::<happen_core::Time>()
        .map(|t| t.delta_f32)
        .unwrap_or(1.0 / 60.0);

    if dt <= 0.0 || dt > 0.1 {
        return;
    }

    apply_forces(world, dt);
    let manifolds = detect_collisions(world);
    resolve_collisions(world, &manifolds, dt);
    integrate(world, dt);
}

fn apply_forces(world: &mut World, _dt: f32) {
    let entities: Vec<_> = world.all_entities();

    let physics_context_exists = world.has_resource::<PhysicsContext>();

    for entity in entities {
        let position = match world.get_component::<Transform>(entity) {
            Some(t) => t.position,
            None => continue,
        };

        let gravity = if physics_context_exists {
            world
                .get_resource::<PhysicsContext>()
                .map(|ctx| ctx.blended_rules_at(position).gravity)
                .unwrap_or(happen_math::Vec3::new(0.0, -9.81, 0.0))
        } else {
            happen_math::Vec3::new(0.0, -9.81, 0.0)
        };

        if let Some(body) = world.get_component_mut::<RigidBody>(entity) {
            if body.body_type != BodyType::Dynamic {
                continue;
            }
            let grav_force = gravity * body.mass * body.gravity_scale;
            body.force_accumulator += grav_force;
        }
    }
}

fn detect_collisions(world: &mut World) -> Vec<ContactManifold> {
    let entities: Vec<_> = world.all_entities();

    let mut collidable: Vec<(happen_core::Entity, Transform, Collider)> = Vec::new();
    for entity in &entities {
        if let (Some(transform), Some(collider)) = (
            world.get_component::<Transform>(*entity),
            world.get_component::<Collider>(*entity),
        ) {
            collidable.push((*entity, *transform, collider.clone()));
        }
    }

    let aabbs: Vec<_> = collidable
        .iter()
        .map(|(e, t, c)| (*e, c.world_aabb(t)))
        .collect();

    let mut broad = BroadPhase::default();
    broad.update(&aabbs);
    let pairs = broad.potential_pairs();

    let mut manifolds = Vec::new();

    for (ea, eb) in pairs {
        let a = collidable.iter().find(|(e, _, _)| *e == ea);
        let b = collidable.iter().find(|(e, _, _)| *e == eb);

        if let (Some((_, ta, ca)), Some((_, tb, cb))) = (a, b) {
            if let Some(manifold) = narrow_phase(ea, ca, ta, eb, cb, tb) {
                manifolds.push(manifold);
            }
        }
    }

    manifolds
}

fn resolve_collisions(world: &mut World, manifolds: &[ContactManifold], dt: f32) {
    if manifolds.is_empty() {
        return;
    }

    let entities: Vec<_> = world.all_entities();
    let mut body_data: Vec<(RigidBody, happen_math::Vec3)> = Vec::new();
    let mut entity_to_index = std::collections::HashMap::new();

    for entity in &entities {
        if let (Some(body), Some(transform)) = (
            world.get_component::<RigidBody>(*entity),
            world.get_component::<Transform>(*entity),
        ) {
            entity_to_index.insert(entity.id(), body_data.len());
            body_data.push((body.clone(), transform.position));
        }
    }

    let solver = ConstraintSolver::default();
    solver.solve(manifolds, &mut body_data, &entity_to_index, dt);

    for entity in &entities {
        if let Some(&idx) = entity_to_index.get(&entity.id()) {
            let (ref solved_body, solved_pos) = body_data[idx];
            if let Some(body) = world.get_component_mut::<RigidBody>(*entity) {
                body.linear_velocity = solved_body.linear_velocity;
            }
            if let Some(transform) = world.get_component_mut::<Transform>(*entity) {
                transform.position = solved_pos;
            }
        }
    }
}

fn integrate(world: &mut World, dt: f32) {
    let max_velocity = world
        .get_resource::<PhysicsConfig>()
        .map(|c| c.max_velocity)
        .unwrap_or(100.0);

    let entities: Vec<_> = world.all_entities();

    for entity in entities {
        let (velocity_delta, damping, inv_mass) = {
            let body = match world.get_component::<RigidBody>(entity) {
                Some(b) if b.body_type == BodyType::Dynamic => b,
                _ => continue,
            };

            let acceleration = body.force_accumulator * body.inverse_mass;
            (acceleration * dt, body.linear_damping, body.inverse_mass)
        };

        if inv_mass <= 0.0 {
            continue;
        }

        if let Some(body) = world.get_component_mut::<RigidBody>(entity) {
            body.linear_velocity += velocity_delta;
            body.linear_velocity *= 1.0 - damping;

            let speed = body.linear_velocity.length();
            if speed > max_velocity {
                body.linear_velocity = body.linear_velocity / speed * max_velocity;
            }

            body.clear_forces();
        }

        let velocity = world
            .get_component::<RigidBody>(entity)
            .map(|b| b.linear_velocity)
            .unwrap_or_default();

        if let Some(transform) = world.get_component_mut::<Transform>(entity) {
            transform.position += velocity * dt;
        }
    }
}
