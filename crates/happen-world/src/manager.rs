use std::collections::HashMap;

use crate::blueprint::{EntityBlueprint, WorldBlueprint};
use crate::terrain::{ChunkCoord, TerrainChunk, TerrainGenerator};
use crate::zone::ZoneDefinition;

pub struct WorldManager {
    zones: Vec<ZoneDefinition>,
    loaded_chunks: HashMap<ChunkCoord, TerrainChunk>,
    terrain_generator: Option<TerrainGenerator>,
    pub load_radius: u32,
    pub blueprint: Option<WorldBlueprint>,
}

impl WorldManager {
    pub fn new() -> Self {
        Self {
            zones: Vec::new(),
            loaded_chunks: HashMap::new(),
            terrain_generator: None,
            load_radius: 3,
            blueprint: None,
        }
    }

    pub fn add_zone(&mut self, zone: ZoneDefinition) {
        if let Some(ref config) = zone.terrain_config {
            self.terrain_generator = Some(TerrainGenerator::new(config.clone()));
        }
        self.zones.push(zone);
    }

    pub fn zones(&self) -> &[ZoneDefinition] {
        &self.zones
    }

    pub fn load_blueprint(&mut self, blueprint: WorldBlueprint) {
        for zone in &blueprint.zones {
            self.add_zone(zone.clone());
        }
        self.blueprint = Some(blueprint);
    }

    pub fn get_chunk(&self, coord: ChunkCoord) -> Option<&TerrainChunk> {
        self.loaded_chunks.get(&coord)
    }

    pub fn update_loaded_chunks(&mut self, camera_x: f32, camera_z: f32, chunk_size: f32) {
        let generator = match &self.terrain_generator {
            Some(g) => g,
            None => return,
        };

        let center_x = (camera_x / chunk_size).floor() as i32;
        let center_z = (camera_z / chunk_size).floor() as i32;
        let radius = self.load_radius as i32;

        let mut needed = Vec::new();
        for x in (center_x - radius)..=(center_x + radius) {
            for z in (center_z - radius)..=(center_z + radius) {
                let coord = ChunkCoord { x, z };
                if !self.loaded_chunks.contains_key(&coord) {
                    needed.push(coord);
                }
            }
        }

        for coord in needed {
            let chunk = generator.generate_chunk(coord);
            self.loaded_chunks.insert(coord, chunk);
        }

        self.loaded_chunks.retain(|coord, _| {
            (coord.x - center_x).abs() <= radius && (coord.z - center_z).abs() <= radius
        });
    }

    pub fn entity_blueprints(&self) -> &[EntityBlueprint] {
        self.blueprint
            .as_ref()
            .map(|b| b.entities.as_slice())
            .unwrap_or(&[])
    }
}

impl Default for WorldManager {
    fn default() -> Self {
        Self::new()
    }
}

