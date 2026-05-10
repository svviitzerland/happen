pub use happen_math;
pub use happen_core;
pub use happen_physics;
pub use happen_render;
pub use happen_world;
pub use happen_ai;

use happen_core::{App, Plugin};
use happen_physics::PhysicsPlugin;
use happen_render::RenderPlugin;
use happen_world::WorldPlugin;
use happen_ai::AiPlugin;

pub mod prelude {
    pub use happen_math::{Vec2, Vec3, Vec4, Mat4, Quat, Transform, Aabb, Ray, Color};
    pub use happen_core::{
        World, Entity, Component, Resource, App, Plugin, Time,
        Events, Event,
        STAGE_FIRST, STAGE_PRE_UPDATE, STAGE_UPDATE, STAGE_POST_UPDATE, STAGE_LAST,
    };
    pub use happen_physics::{
        RigidBody, Collider, ColliderShape, BodyType,
        WorldRules, PhysicsZone, PhysicsContext, PhysicsConfig,
    };
    pub use happen_render::{
        Camera, Projection, MeshRenderer, Mesh, Material,
        MeshHandle, MaterialHandle, MeshAssets, MaterialAssets,
    };
    pub use happen_world::{
        WorldManager, ZoneDefinition, TerrainConfig, WorldBlueprint, EntityBlueprint,
        EnvironmentConfig,
    };
    pub use happen_ai::{AiOrchestrator, UserIntent, BlueprintApplicator};
    pub use crate::HappenEngine;
}

pub struct HappenEngine;

impl HappenEngine {
    pub fn full() -> App {
        let mut app = App::new();
        app.add_plugin(PhysicsPlugin);
        app.add_plugin(WorldPlugin);
        app.add_plugin(AiPlugin::default());
        app.add_plugin(RenderPlugin);
        app
    }

    pub fn headless() -> App {
        let mut app = App::new();
        app.add_plugin(PhysicsPlugin);
        app.add_plugin(WorldPlugin);
        app.add_plugin(AiPlugin::default());
        app
    }

    pub fn from_blueprint(blueprint: happen_world::WorldBlueprint) -> App {
        let mut app = Self::full();
        app.world.insert_resource(
            happen_world::WorldManager::new()
        );
        {
            if let Some(mgr) = app.world.get_resource_mut::<happen_world::WorldManager>() {
                mgr.load_blueprint(blueprint);
            }
        }
        app
    }
}

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, _app: &mut App) {}

    fn name(&self) -> &str {
        "CorePlugin"
    }
}
