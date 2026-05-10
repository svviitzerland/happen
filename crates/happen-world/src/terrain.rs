use happen_math::Vec3;
use noise::{NoiseFn, Perlin, Simplex};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum NoiseType {
    Perlin,
    Simplex,
    Ridged,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum BlendMode {
    Add,
    Multiply,
    Max,
    Min,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NoiseLayer {
    pub noise_type: NoiseType,
    pub frequency: f64,
    pub amplitude: f64,
    pub octaves: u32,
    pub lacunarity: f64,
    pub persistence: f64,
    pub offset: Vec3,
    pub blend_mode: BlendMode,
}

impl Default for NoiseLayer {
    fn default() -> Self {
        Self {
            noise_type: NoiseType::Perlin,
            frequency: 0.01,
            amplitude: 10.0,
            octaves: 4,
            lacunarity: 2.0,
            persistence: 0.5,
            offset: Vec3::ZERO,
            blend_mode: BlendMode::Add,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TerrainConfig {
    pub seed: u64,
    pub chunk_size: f32,
    pub resolution: u32,
    pub height_scale: f32,
    pub layers: Vec<NoiseLayer>,
}

impl Default for TerrainConfig {
    fn default() -> Self {
        Self {
            seed: 42,
            chunk_size: 64.0,
            resolution: 32,
            height_scale: 20.0,
            layers: vec![NoiseLayer::default()],
        }
    }
}

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ChunkCoord {
    pub x: i32,
    pub z: i32,
}

pub struct TerrainChunk {
    pub coord: ChunkCoord,
    pub heightmap: Vec<f32>,
    pub resolution: u32,
}

pub struct TerrainGenerator {
    config: TerrainConfig,
    perlin: Perlin,
    simplex: Simplex,
}

impl TerrainGenerator {
    pub fn new(config: TerrainConfig) -> Self {
        let perlin = Perlin::new(config.seed as u32);
        let simplex = Simplex::new(config.seed as u32);
        Self {
            config,
            perlin,
            simplex,
        }
    }

    pub fn height_at(&self, world_x: f32, world_z: f32) -> f32 {
        let mut total = 0.0f64;

        for layer in &self.config.layers {
            let mut layer_value = 0.0f64;
            let mut freq = layer.frequency;
            let mut amp = layer.amplitude;

            let x = world_x as f64 + layer.offset.x as f64;
            let z = world_z as f64 + layer.offset.z as f64;

            for _ in 0..layer.octaves {
                let sample = match layer.noise_type {
                    NoiseType::Perlin => self.perlin.get([x * freq, z * freq]),
                    NoiseType::Simplex => self.simplex.get([x * freq, z * freq]),
                    NoiseType::Ridged => {
                        let v = self.perlin.get([x * freq, z * freq]);
                        1.0 - (v.abs() * 2.0)
                    }
                };

                layer_value += sample * amp;
                freq *= layer.lacunarity;
                amp *= layer.persistence;
            }

            total = match layer.blend_mode {
                BlendMode::Add => total + layer_value,
                BlendMode::Multiply => total * layer_value,
                BlendMode::Max => total.max(layer_value),
                BlendMode::Min => total.min(layer_value),
            };
        }

        total as f32 * self.config.height_scale
    }

    pub fn generate_chunk(&self, coord: ChunkCoord) -> TerrainChunk {
        let res = self.config.resolution;
        let size = self.config.chunk_size;
        let step = size / res as f32;
        let origin_x = coord.x as f32 * size;
        let origin_z = coord.z as f32 * size;

        let mut heightmap = Vec::with_capacity((res + 1) as usize * (res + 1) as usize);

        for z in 0..=res {
            for x in 0..=res {
                let wx = origin_x + x as f32 * step;
                let wz = origin_z + z as f32 * step;
                heightmap.push(self.height_at(wx, wz));
            }
        }

        TerrainChunk {
            coord,
            heightmap,
            resolution: res,
        }
    }
}
