use happen_world::WorldBlueprint;
use serde::{Deserialize, Serialize};

use crate::provider::{AiError, AiProvider};

const SYSTEM_PROMPT: &str = r#"You are Happen Engine's world generator AI. Given a user's description of a game world, generate a WorldBlueprint JSON.

The WorldBlueprint schema:
{
  "name": "string - world name",
  "description": "string - world description",
  "zones": [
    {
      "id": "string - unique zone id",
      "name": "string - display name",
      "bounds": { "min": [x, y, z], "max": [x, y, z] },
      "terrain_config": null or {
        "seed": number,
        "chunk_size": 64.0,
        "resolution": 32,
        "height_scale": number,
        "layers": [{ "noise_type": "Perlin"|"Simplex"|"Ridged", "frequency": number, "amplitude": number, "octaves": number, "lacunarity": 2.0, "persistence": 0.5, "offset": [0,0,0], "blend_mode": "Add"|"Multiply"|"Max"|"Min" }]
      },
      "physics_rules": {
        "gravity": [x, y, z],
        "air_density": number,
        "terminal_velocity": number or null,
        "time_scale": 1.0,
        "friction_modifier": 1.0,
        "restitution_modifier": 1.0
      },
      "environment": {
        "ambient_light_color": { "r": 1, "g": 1, "b": 1, "a": 1 },
        "ambient_light_intensity": 0.3,
        "fog_color": { "r": 0.7, "g": 0.8, "b": 0.9, "a": 1 },
        "fog_density": 0.001,
        "sky_color": { "r": 0.4, "g": 0.6, "b": 0.9, "a": 1 }
      },
      "priority": number,
      "blend_margin": number
    }
  ],
  "spawn_point": [x, y, z],
  "entities": [
    {
      "name": "string",
      "transform": { "position": [x, y, z], "rotation": [x, y, z, w], "scale": [x, y, z] },
      "mesh_type": "cube"|"sphere"|"plane",
      "color": { "r": 0-1, "g": 0-1, "b": 0-1, "a": 1 },
      "metallic": 0-1,
      "roughness": 0-1,
      "scale": [x, y, z],
      "physics": null or { "body_type": "dynamic"|"static"|"kinematic", "collider_shape": "box"|"sphere"|"capsule", "mass": number, "restitution": 0-1, "friction": 0-1 },
      "tags": ["string"]
    }
  ]
}

Physics presets for gravity:
- Earth: [0, -9.81, 0], air_density: 1.225
- Moon: [0, -1.62, 0], air_density: 0
- Mars: [0, -3.72, 0], air_density: 0.02
- Space: [0, 0, 0], air_density: 0
- Underwater: [0, -2.0, 0], air_density: 1000

Rules:
1. Output ONLY valid JSON, no markdown code blocks
2. Create a rich world with multiple entities
3. Use realistic physics rules based on environment
4. Place entities at sensible positions
5. Use varied colors and materials
6. Set spawn_point where the player camera should start
7. Scale objects realistically
"#;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserIntent {
    pub prompt: String,
    pub style: Option<String>,
    pub constraints: Vec<String>,
}

impl UserIntent {
    pub fn new(prompt: &str) -> Self {
        Self {
            prompt: prompt.to_string(),
            style: None,
            constraints: Vec::new(),
        }
    }
}

pub struct AiOrchestrator {
    provider: Box<dyn AiProvider>,
}

impl AiOrchestrator {
    pub fn new(provider: Box<dyn AiProvider>) -> Self {
        Self { provider }
    }

    pub async fn generate_world(&self, intent: &UserIntent) -> Result<WorldBlueprint, AiError> {
        let mut prompt = format!("Generate a game world: {}", intent.prompt);

        if let Some(ref style) = intent.style {
            prompt.push_str(&format!("\nStyle: {}", style));
        }

        for constraint in &intent.constraints {
            prompt.push_str(&format!("\nConstraint: {}", constraint));
        }

        let response = self.provider.generate(&prompt, SYSTEM_PROMPT).await?;

        let cleaned = response
            .trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();

        serde_json::from_str(cleaned)
            .map_err(|e| AiError::Parse(format!("Failed to parse WorldBlueprint: {}. Response: {}", e, &cleaned[..cleaned.len().min(500)])))
    }
}
