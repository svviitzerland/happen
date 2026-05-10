use happen_math::{Aabb, Color};
use happen_physics::WorldRules;
use serde::{Deserialize, Serialize};

use crate::terrain::TerrainConfig;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnvironmentConfig {
    pub ambient_light_color: Color,
    pub ambient_light_intensity: f32,
    pub fog_color: Color,
    pub fog_density: f32,
    pub sky_color: Color,
}

impl Default for EnvironmentConfig {
    fn default() -> Self {
        Self {
            ambient_light_color: Color::WHITE,
            ambient_light_intensity: 0.3,
            fog_color: Color::new(0.7, 0.8, 0.9, 1.0),
            fog_density: 0.001,
            sky_color: Color::CORNFLOWER_BLUE,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ZoneDefinition {
    pub id: String,
    pub name: String,
    pub bounds: Aabb,
    pub terrain_config: Option<TerrainConfig>,
    pub physics_rules: WorldRules,
    pub environment: EnvironmentConfig,
    pub priority: i32,
    pub blend_margin: f32,
}

impl ZoneDefinition {
    pub fn to_physics_zone(&self) -> happen_physics::PhysicsZone {
        happen_physics::PhysicsZone {
            name: self.id.clone(),
            bounds: self.bounds,
            rules: self.physics_rules.clone(),
            priority: self.priority,
            blend_margin: self.blend_margin,
        }
    }
}
