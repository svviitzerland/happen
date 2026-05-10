mod terrain;
mod blueprint;
mod zone;
mod manager;

pub use terrain::{TerrainConfig, NoiseLayer, NoiseType, BlendMode, TerrainChunk, TerrainGenerator, ChunkCoord};
pub use blueprint::{WorldBlueprint, EntityBlueprint, PhysicsBlueprint};
pub use zone::{ZoneDefinition, EnvironmentConfig};
pub use manager::WorldManager;

use happen_core::{App, Plugin};

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WorldManager::new());
    }

    fn name(&self) -> &str {
        "WorldPlugin"
    }
}
