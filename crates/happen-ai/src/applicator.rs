use happen_core::World;
use happen_physics::{Collider, PhysicsContext, RigidBody};
use happen_world::{EntityBlueprint, WorldBlueprint};

pub struct BlueprintApplicator;

impl BlueprintApplicator {
    pub fn apply(world: &mut World, blueprint: &WorldBlueprint) {
        for zone in &blueprint.zones {
            if let Some(ctx) = world.get_resource_mut::<PhysicsContext>() {
                ctx.add_zone(zone.to_physics_zone());
            }
        }

        for entity_bp in &blueprint.entities {
            Self::spawn_entity(world, entity_bp);
        }

        log::info!(
            "Applied blueprint '{}': {} zones, {} entities",
            blueprint.name,
            blueprint.zones.len(),
            blueprint.entities.len()
        );
    }

    pub fn spawn_entity(world: &mut World, bp: &EntityBlueprint) -> happen_core::Entity {
        let mut transform = bp.transform;
        transform.scale = bp.scale;

        let builder = world.spawn();
        let entity = builder.id();
        drop(builder);

        world.insert_component(entity, transform);

        if let Some(ref phys) = bp.physics {
            let body = match phys.body_type.as_str() {
                "dynamic" => {
                    let mut b = RigidBody::dynamic(phys.mass);
                    b.restitution = phys.restitution;
                    b.friction = phys.friction;
                    b
                }
                "kinematic" => RigidBody::kinematic(),
                _ => RigidBody::statik(),
            };
            world.insert_component(entity, body);

            let collider = match phys.collider_shape.as_str() {
                "sphere" => Collider::sphere(bp.scale.x * 0.5),
                "capsule" => Collider::capsule(bp.scale.x * 0.5, bp.scale.y * 0.5),
                _ => Collider::cuboid(bp.scale * 0.5),
            };
            world.insert_component(entity, collider);
        }

        entity
    }
}

pub fn mesh_type_to_handle(mesh_type: &str) -> usize {
    match mesh_type {
        "cube" => 0,
        "sphere" => 1,
        "plane" => 2,
        _ => 0,
    }
}

pub fn spawn_blueprint_with_rendering(
    world: &mut World,
    blueprint: &WorldBlueprint,
    upload_material: &dyn Fn(&happen_math::Color, f32, f32) -> usize,
) {
    for zone in &blueprint.zones {
        if let Some(ctx) = world.get_resource_mut::<PhysicsContext>() {
            ctx.add_zone(zone.to_physics_zone());
        }
    }

    for bp in &blueprint.entities {
        let entity = BlueprintApplicator::spawn_entity(world, bp);

        let mesh_handle = mesh_type_to_handle(&bp.mesh_type);
        let material_handle = upload_material(&bp.color, bp.metallic, bp.roughness);

        use happen_render::MeshRenderer;
        world.insert_component(entity, MeshRenderer::new(mesh_handle, material_handle));
    }
}
