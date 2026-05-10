pub use happen_ai;
pub use happen_core;
pub use happen_math;
pub use happen_physics;
pub use happen_render;
pub use happen_world;

use happen_ai::BlueprintApplicator;
use happen_core::App;
use happen_math::{Quat, Transform, Vec3};
use happen_physics::PhysicsPlugin;
use happen_render::{
    run_with_init, Camera, Material, MaterialAssets, MeshAssets, MeshRenderer, Projection,
    RenderPlugin,
};
use happen_world::{WorldBlueprint, WorldManager, WorldPlugin};

pub mod prelude {
    pub use happen_ai::{AiOrchestrator, BlueprintApplicator, UserIntent};
    pub use happen_core::{
        App, Component, Entity, Event, Events, Plugin, Resource, Time, World, STAGE_FIRST,
        STAGE_LAST, STAGE_POST_UPDATE, STAGE_PRE_UPDATE, STAGE_UPDATE,
    };
    pub use happen_math::{Aabb, Color, Mat4, Quat, Ray, Transform, Vec2, Vec3, Vec4};
    pub use happen_physics::{
        BodyType, Collider, ColliderShape, PhysicsConfig, PhysicsContext, PhysicsZone, RigidBody,
        WorldRules,
    };
    pub use happen_render::{
        Camera, Material, MaterialAssets, MaterialHandle, Mesh, MeshAssets, MeshHandle,
        MeshRenderer, Projection,
    };
    pub use happen_world::{
        EntityBlueprint, EnvironmentConfig, TerrainConfig, WorldBlueprint, WorldManager,
        ZoneDefinition,
    };

    pub use crate::HappenEngine;
}

pub struct HappenEngine;

impl HappenEngine {
    pub fn full() -> App {
        let mut app = App::new();
        app.add_plugin(PhysicsPlugin);
        app.add_plugin(WorldPlugin);
        app.add_plugin(happen_ai::AiPlugin::default());
        app.add_plugin(RenderPlugin);
        app.set_runner(|app| {
            run_with_init(app, happen_render::default_init_callback());
        });
        app
    }

    pub fn headless() -> App {
        let mut app = App::new();
        app.add_plugin(PhysicsPlugin);
        app.add_plugin(WorldPlugin);
        app.add_plugin(happen_ai::AiPlugin::default());
        app
    }

    pub fn from_blueprint(blueprint: WorldBlueprint) -> App {
        let mut app = App::new();
        app.add_plugin(PhysicsPlugin);
        app.add_plugin(WorldPlugin);
        app.add_plugin(happen_ai::AiPlugin::default());
        app.add_plugin(RenderPlugin);

        let spawn_point = blueprint.spawn_point;

        let mut mgr = WorldManager::new();
        mgr.load_blueprint(blueprint);
        app.world.insert_resource(mgr);

        app.set_runner(move |app| {
            run_with_init(
                app,
                Box::new(move |gpu, render_state, app| {
                    let mut mesh_assets = MeshAssets::new();
                    let mut material_assets = MaterialAssets::new();

                    let cube_mesh = happen_render::Mesh::cube(1.0);
                    mesh_assets.upload(&gpu.device, &cube_mesh);

                    let sphere_mesh = happen_render::Mesh::sphere(0.5, 32, 16);
                    mesh_assets.upload(&gpu.device, &sphere_mesh);

                    let plane_mesh = happen_render::Mesh::plane(100.0, 100.0);
                    mesh_assets.upload(&gpu.device, &plane_mesh);

                    let default_mat = Material::default();
                    material_assets.upload(
                        &gpu.device,
                        &render_state.material_bind_group_layout,
                        &default_mat,
                    );

                    let blueprints: Vec<_> = app
                        .world
                        .get_resource::<WorldManager>()
                        .map(|mgr| mgr.entity_blueprints().to_vec())
                        .unwrap_or_default();

                    for bp in &blueprints {
                        let entity = BlueprintApplicator::spawn_entity(&mut app.world, bp);

                        let mesh_handle = BlueprintApplicator::mesh_type_to_handle(&bp.mesh_type);

                        let mat = Material::metallic(bp.color, bp.metallic, bp.roughness);
                        let mat_handle = material_assets.upload(
                            &gpu.device,
                            &render_state.material_bind_group_layout,
                            &mat,
                        );

                        app.world
                            .insert_component(entity, MeshRenderer::new(mesh_handle, mat_handle));
                    }

                    let look_target = Vec3::ZERO;
                    let look_dir = (look_target - spawn_point).normalize_or_zero();
                    let cam_rotation = if look_dir.length_squared() > 0.001 {
                        Quat::from_rotation_arc(-Vec3::Z, look_dir)
                    } else {
                        Quat::IDENTITY
                    };

                    let cam_entity = app.world.spawn_empty();
                    app.world.insert_component(
                        cam_entity,
                        Transform::from_position_rotation(spawn_point, cam_rotation),
                    );
                    app.world.insert_component(
                        cam_entity,
                        Camera::new(Projection::perspective(
                            60.0_f32.to_radians(),
                            1280.0 / 720.0,
                            0.1,
                            1000.0,
                        )),
                    );

                    app.world.insert_resource(mesh_assets);
                    app.world.insert_resource(material_assets);

                    log::info!(
                        "Loaded {} entities, camera at {:?}",
                        blueprints.len(),
                        spawn_point
                    );
                }),
            );
        });

        app
    }
}
