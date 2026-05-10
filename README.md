# Happen Engine

> AI-first game engine. Describe the game you want — AI builds it.

Happen is a lightweight, high-performance game engine written in Rust. Instead of manually placing assets and writing game logic, you describe your world in natural language and the engine's AI layer generates everything: terrain, entities, physics rules, and materials.

## Features

- **AI World Generation** — Describe your game in plain text. The AI generates a complete `WorldBlueprint` with terrain, entities, physics, and materials.
- **Context-Aware Physics** — The `WorldRules` system lets each zone define its own physical laws. Earth gravity, lunar gravity, zero-G space, underwater — all in the same world, with smooth blending at zone boundaries.
- **Custom ECS** — SparseSet-based Entity Component System. O(1) entity lookup, cache-friendly iteration, no external ECS dependency.
- **wgpu Renderer** — Cross-platform GPU rendering (Vulkan, Metal, DX12, WebGPU). Blinn-Phong lighting, depth buffer, primitive mesh generation.
- **Procedural Terrain** — Noise-based terrain generation with configurable layers (Perlin, Simplex, Ridged), chunk streaming, and per-zone terrain configs.
- **Blueprint System** — Serialize entire worlds as JSON. Load, inspect, share, and version control your game worlds.
- **Plugin Architecture** — Every subsystem (physics, rendering, world, AI) is a composable plugin. Use only what you need.

## Quick Start

```bash
# Run the demo scene
cargo run --bin happen -- demo

# Generate a world from a text prompt
ANTHROPIC_API_KEY=your_key cargo run --bin happen -- generate "medieval village with a castle on a hill" -o world.json

# Load and run a blueprint
cargo run --bin happen -- blueprint world.json

# Inspect a blueprint
cargo run --bin happen -- inspect world.json

# Create a new game project
cargo run --bin happen -- new my_game
```

## Architecture

```
happen/
├── crates/
│   ├── happen-math       Tier 0   Math types, transforms, geometry (wraps glam)
│   ├── happen-core       Tier 0   ECS, App framework, system scheduling, events
│   ├── happen-physics    Tier 1   Rigid body physics, WorldRules, collision detection
│   ├── happen-render     Tier 1   wgpu renderer, camera, materials, mesh primitives
│   ├── happen-world      Tier 2   Terrain generation, zones, blueprint serialization
│   ├── happen-ai         Tier 2   AI orchestration, LLM providers, blueprint applicator
│   └── happen-engine     Tier 3   Glue crate, prelude, engine builders
└── happen-cli            Tier 4   CLI binary
```

## WorldRules — Context-Aware Physics

The core differentiator. Each zone in your world can have completely different physical laws:

```rust
use happen_engine::prelude::*;

// Earth zone
let earth = WorldRules::earth();       // gravity: -9.81 m/s², air density: 1.225

// Space station zone
let space = WorldRules::zero_g();      // gravity: 0, no air resistance

// Moon surface zone  
let moon = WorldRules::lunar();        // gravity: -1.62 m/s²

// Underwater cave zone
let underwater = WorldRules::underwater(); // gravity: -2.0, high drag, slow time
```

Zones blend smoothly at boundaries — an object floating from space into a planet's gravity well experiences a gradual transition, not a sudden snap.

## AI World Generation

With an Anthropic API key, you can generate entire worlds from text:

```bash
happen generate "open world RPG with floating islands connected by bridges, \
  each island has different gravity" -o floating_world.json
```

The AI understands physics contexts and will assign appropriate `WorldRules` to each zone automatically.

## Using as a Library

```rust
use happen_engine::prelude::*;

fn main() {
    let mut app = HappenEngine::full();

    // Add custom systems
    app.add_system(STAGE_UPDATE, "my_system", |world: &mut World| {
        // your game logic here
    });

    // Spawn entities
    let entity = app.world.spawn()
        .with(Transform::from_position(Vec3::new(0.0, 5.0, 0.0)))
        .with(RigidBody::dynamic(1.0))
        .with(Collider::sphere(0.5))
        .build();

    app.run();
}
```

## Blueprint Format

Worlds are serialized as JSON for easy editing, sharing, and version control:

```json
{
  "name": "My World",
  "description": "A demo world",
  "spawn_point": [0, 5, 10],
  "zones": [
    {
      "id": "earth_zone",
      "name": "Earth Surface",
      "bounds": { "min": [-500, -10, -500], "max": [500, 200, 500] },
      "physics_rules": {
        "gravity": [0, -9.81, 0],
        "air_density": 1.225
      }
    }
  ],
  "entities": [
    {
      "name": "Red Cube",
      "transform": { "position": [0, 5, 0], "rotation": [0, 0, 0, 1], "scale": [1, 1, 1] },
      "mesh_type": "cube",
      "color": { "r": 1, "g": 0, "b": 0, "a": 1 },
      "physics": { "body_type": "dynamic", "collider_shape": "box", "mass": 1.0 }
    }
  ]
}
```

## Requirements

- Rust 1.75+
- GPU with Vulkan, Metal, or DX12 support

## License

MIT
