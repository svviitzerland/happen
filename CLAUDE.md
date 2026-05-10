# Happen Engine

AI-first AAA game engine built in Rust.

## Architecture

Cargo workspace with 8 crates, layered by dependency:

- **happen-math** (Tier 0) — Vec3, Mat4, Transform, AABB, Ray, Color. Wraps glam.
- **happen-core** (Tier 0) — Custom ECS (SparseSet storage), App/Plugin framework, system scheduling, events, Time resource.
- **happen-physics** (Tier 1) — Context-aware physics with WorldRules system. Sequential impulse solver, broad/narrow phase collision.
- **happen-render** (Tier 1) — wgpu-based renderer. Blinn-Phong lighting, depth buffer, mesh primitives.
- **happen-world** (Tier 2) — Procedural terrain (noise-based), zone definitions, WorldBlueprint serialization format.
- **happen-ai** (Tier 2) — AI orchestration via Anthropic API. Text prompt → WorldBlueprint JSON → live entities.
- **happen-engine** (Tier 3) — Glue crate. `HappenEngine::full()` / `::headless()` builders, prelude re-exports.
- **happen-cli** (Tier 4) — Binary. Commands: `demo`, `blueprint`, `generate`, `inspect`, `new`.

## Key Concepts

- **WorldRules** — Each physics zone defines its own gravity, air density, terminal velocity, friction/restitution modifiers. Zones blend smoothly at boundaries.
- **WorldBlueprint** — JSON format describing a complete game world (zones, entities, physics, materials). AI generates these; engine consumes them.
- **ECS uses blanket impls** — Any `Send + Sync + 'static` type is automatically a Component and Resource. No manual impl needed.

## Build & Run

```bash
cargo build                          # build everything
cargo run --bin happen -- demo       # run demo scene
cargo run --bin happen -- --help     # see all commands
cargo check                          # fast type check
```

## Code Conventions

- Rust 2021 edition
- No external ECS crate — custom SparseSet-based ECS in happen-core
- Math types re-exported through happen-math, never import glam directly from other crates
- Plugin pattern: each subsystem implements `Plugin` trait with `build(&self, app: &mut App)`
- Systems are `FnMut(&mut World) + Send + Sync` closures registered to named stages (first → pre_update → update → post_update → last)
- Shader source in `crates/happen-render/src/shaders/`, embedded via `include_str!`
