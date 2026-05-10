use clap::{Parser, Subcommand};
use happen_engine::prelude::*;
use happen_engine::{happen_physics, happen_render, happen_core};

#[derive(Parser)]
#[command(name = "happen")]
#[command(about = "Happen - AI-First Game Engine")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the engine with a demo scene
    Demo,

    /// Load and run a world from a blueprint JSON file
    Blueprint {
        /// Path to the blueprint JSON file
        file: String,
    },

    /// Generate a world from an AI prompt
    Generate {
        /// Description of the world to generate
        prompt: String,

        /// Save the generated blueprint to a file
        #[arg(short, long)]
        output: Option<String>,

        /// AI provider: anthropic or openrouter
        #[arg(short, long, default_value = "anthropic")]
        provider: String,

        /// Model to use (e.g. claude-sonnet-4-6, google/gemini-2.5-flash)
        #[arg(short, long)]
        model: Option<String>,
    },

    /// Inspect a blueprint file
    Inspect {
        /// Path to the blueprint JSON file
        file: String,
    },

    /// Create a new game project
    New {
        /// Project name
        name: String,
    },
}

fn main() {
    env_logger::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::Demo => run_demo(),
        Commands::Blueprint { file } => run_blueprint(&file),
        Commands::Generate { prompt, output, provider, model } => {
            generate_world(&prompt, output, &provider, model)
        }
        Commands::Inspect { file } => inspect_blueprint(&file),
        Commands::New { name } => create_project(&name),
    }
}

fn run_demo() {
    println!("Happen Engine v{}", env!("CARGO_PKG_VERSION"));
    println!("Starting playable demo...");
    println!();
    println!("  Controls:");
    println!("    Click     - Lock mouse / enable controls");
    println!("    WASD      - Move");
    println!("    Mouse     - Look around");
    println!("    Space     - Jump");
    println!("    Shift     - Sprint");
    println!("    Escape    - Release mouse");
    println!();

    let mut app = happen_core::App::new();
    app.add_plugin(happen_physics::PhysicsPlugin);
    app.add_plugin(happen_world::WorldPlugin);
    app.add_plugin(happen_render::RenderPlugin);

    app.set_runner(|app| {
        happen_render::run_with_init(
            app,
            Box::new(|gpu, render_state, app| {
                use happen_render::*;

                let mut mesh_assets = MeshAssets::new();
                let mut material_assets = MaterialAssets::new();

                let cube = Mesh::cube(1.0);
                let cube_h = mesh_assets.upload(&gpu.device, &cube);

                let sphere = Mesh::sphere(0.5, 32, 16);
                let sphere_h = mesh_assets.upload(&gpu.device, &sphere);

                let plane = Mesh::plane(200.0, 200.0);
                let plane_h = mesh_assets.upload(&gpu.device, &plane);

                use happen_math::Color;
                let layout = &render_state.material_bind_group_layout;
                let dev = &gpu.device;

                let ground_h = material_assets.upload(dev, layout, &Material::new(Color::new(0.3, 0.5, 0.2, 1.0)));
                let wall_h = material_assets.upload(dev, layout, &Material::new(Color::new(0.6, 0.6, 0.55, 1.0)));
                let red_h = material_assets.upload(dev, layout, &Material::new(Color::RED));
                let blue_h = material_assets.upload(dev, layout, &Material::metallic(Color::BLUE, 0.5, 0.3));
                let yellow_h = material_assets.upload(dev, layout, &Material::new(Color::YELLOW));
                let orange_h = material_assets.upload(dev, layout, &Material::new(Color::new(1.0, 0.5, 0.1, 1.0)));
                let white_h = material_assets.upload(dev, layout, &Material::metallic(Color::WHITE, 0.8, 0.2));
                let dark_h = material_assets.upload(dev, layout, &Material::new(Color::new(0.25, 0.25, 0.3, 1.0)));
                let cyan_h = material_assets.upload(dev, layout, &Material::metallic(Color::new(0.1, 0.8, 0.9, 1.0), 0.6, 0.2));
                let purple_h = material_assets.upload(dev, layout, &Material::new(Color::new(0.6, 0.2, 0.8, 1.0)));

                app.world.insert_resource(mesh_assets);
                app.world.insert_resource(material_assets);

                let spawn_obj =
                    |world: &mut happen_core::World,
                     pos: Vec3,
                     scale: Vec3,
                     mesh: MeshHandle,
                     mat: MaterialHandle| {
                        let e = world.spawn_empty();
                        world.insert_component(
                            e,
                            Transform {
                                position: pos,
                                scale,
                                ..Transform::IDENTITY
                            },
                        );
                        world.insert_component(e, MeshRenderer::new(mesh, mat));
                    };

                // Ground
                spawn_obj(
                    &mut app.world,
                    Vec3::ZERO,
                    Vec3::ONE,
                    plane_h,
                    ground_h,
                );

                // === Central plaza ===
                // Red pillar
                spawn_obj(
                    &mut app.world,
                    Vec3::new(0.0, 2.5, 0.0),
                    Vec3::new(1.0, 5.0, 1.0),
                    cube_h,
                    red_h,
                );
                // Metallic sphere on top
                spawn_obj(
                    &mut app.world,
                    Vec3::new(0.0, 5.5, 0.0),
                    Vec3::splat(2.0),
                    sphere_h,
                    white_h,
                );

                // === Building 1 (right side) ===
                spawn_obj(
                    &mut app.world,
                    Vec3::new(12.0, 3.0, -5.0),
                    Vec3::new(6.0, 6.0, 8.0),
                    cube_h,
                    wall_h,
                );
                spawn_obj(
                    &mut app.world,
                    Vec3::new(12.0, 7.0, -5.0),
                    Vec3::new(7.0, 1.0, 9.0),
                    cube_h,
                    dark_h,
                );

                // === Building 2 (left side, tall) ===
                spawn_obj(
                    &mut app.world,
                    Vec3::new(-10.0, 5.0, -8.0),
                    Vec3::new(5.0, 10.0, 5.0),
                    cube_h,
                    wall_h,
                );
                spawn_obj(
                    &mut app.world,
                    Vec3::new(-10.0, 11.0, -8.0),
                    Vec3::new(6.0, 1.0, 6.0),
                    cube_h,
                    dark_h,
                );

                // === Corridor walls ===
                spawn_obj(
                    &mut app.world,
                    Vec3::new(4.0, 1.5, 8.0),
                    Vec3::new(12.0, 3.0, 0.5),
                    cube_h,
                    wall_h,
                );
                spawn_obj(
                    &mut app.world,
                    Vec3::new(4.0, 1.5, 14.0),
                    Vec3::new(12.0, 3.0, 0.5),
                    cube_h,
                    wall_h,
                );

                // === Scattered objects ===
                // Orange crates
                spawn_obj(
                    &mut app.world,
                    Vec3::new(6.0, 0.5, 3.0),
                    Vec3::splat(1.0),
                    cube_h,
                    orange_h,
                );
                spawn_obj(
                    &mut app.world,
                    Vec3::new(6.5, 1.5, 3.2),
                    Vec3::splat(0.8),
                    cube_h,
                    orange_h,
                );
                spawn_obj(
                    &mut app.world,
                    Vec3::new(7.5, 0.5, 2.5),
                    Vec3::splat(1.0),
                    cube_h,
                    orange_h,
                );

                // Blue spheres
                spawn_obj(
                    &mut app.world,
                    Vec3::new(-5.0, 1.0, 4.0),
                    Vec3::splat(2.0),
                    sphere_h,
                    blue_h,
                );
                spawn_obj(
                    &mut app.world,
                    Vec3::new(-3.0, 0.6, 6.0),
                    Vec3::splat(1.2),
                    sphere_h,
                    cyan_h,
                );

                // Yellow marker
                spawn_obj(
                    &mut app.world,
                    Vec3::new(0.0, 0.3, 11.0),
                    Vec3::new(0.6, 0.6, 0.6),
                    sphere_h,
                    yellow_h,
                );

                // Purple tower far away
                spawn_obj(
                    &mut app.world,
                    Vec3::new(-20.0, 4.0, -20.0),
                    Vec3::new(3.0, 8.0, 3.0),
                    cube_h,
                    purple_h,
                );
                spawn_obj(
                    &mut app.world,
                    Vec3::new(-20.0, 9.0, -20.0),
                    Vec3::splat(4.0),
                    sphere_h,
                    purple_h,
                );

                // === Ramp / stairs (stacked cubes) ===
                for i in 0..5 {
                    let y = i as f32 * 0.4 + 0.2;
                    let z = -2.0 - i as f32 * 1.0;
                    spawn_obj(
                        &mut app.world,
                        Vec3::new(20.0, y, z),
                        Vec3::new(3.0, 0.4, 1.0),
                        cube_h,
                        dark_h,
                    );
                }

                // === FPS Camera / Player ===
                let player = app.world.spawn_empty();
                app.world.insert_component(
                    player,
                    Transform::from_position(Vec3::new(0.0, 1.7, 10.0)),
                );
                app.world.insert_component(
                    player,
                    Camera::new(Projection::perspective(
                        70.0_f32.to_radians(),
                        1280.0 / 720.0,
                        0.1,
                        500.0,
                    )),
                );
                app.world
                    .insert_component(player, FpsController::default());

                println!("Loaded playable demo: {} entities", app.world.all_entities().len());
            }),
        );
    });

    app.run();
}

fn run_blueprint(path: &str) {
    let contents = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", path, e);
            std::process::exit(1);
        }
    };

    let blueprint: WorldBlueprint = match serde_json::from_str(&contents) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Error parsing blueprint: {}", e);
            std::process::exit(1);
        }
    };

    println!("Happen Engine - Loading '{}'", blueprint.name);
    println!("  Zones: {}", blueprint.zones.len());
    println!("  Entities: {}", blueprint.entities.len());

    let app = HappenEngine::from_blueprint(blueprint);
    app.run();
}

fn generate_world(prompt: &str, output: Option<String>, provider_name: &str, model: Option<String>) {
    println!("Happen Engine - AI World Generator");
    println!("Provider: {}", provider_name);
    println!("Prompt: \"{}\"", prompt);

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let ai_provider: Box<dyn happen_ai::AiProvider> = match provider_name {
            "openrouter" => {
                let config = happen_ai::AiProviderConfig {
                    provider: "openrouter".to_string(),
                    model: model.unwrap_or_else(|| "anthropic/claude-sonnet-4".to_string()),
                    api_key: None,
                    base_url: None,
                };
                match happen_ai::OpenRouterProvider::from_config(&config) {
                    Ok(p) => Box::new(p),
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        eprintln!("Set OPENROUTER_API_KEY environment variable.");
                        fallback_demo(&output);
                        return;
                    }
                }
            }
            _ => {
                let config = happen_ai::AiProviderConfig {
                    provider: "anthropic".to_string(),
                    model: model.unwrap_or_else(|| "claude-sonnet-4-6".to_string()),
                    api_key: None,
                    base_url: None,
                };
                match happen_ai::AnthropicProvider::from_config(&config) {
                    Ok(p) => Box::new(p),
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        eprintln!("Set ANTHROPIC_API_KEY environment variable.");
                        fallback_demo(&output);
                        return;
                    }
                }
            }
        };

        let orchestrator = happen_ai::AiOrchestrator::new(ai_provider);
        let intent = happen_ai::UserIntent::new(prompt);

        println!("Generating world with AI...");

        match orchestrator.generate_world(&intent).await {
            Ok(blueprint) => {
                println!("Generated: '{}'", blueprint.name);
                println!("  {} zones, {} entities", blueprint.zones.len(), blueprint.entities.len());

                let json = serde_json::to_string_pretty(&blueprint).unwrap();
                if let Some(ref path) = output {
                    std::fs::write(path, &json).unwrap();
                    println!("Saved to: {}", path);
                } else {
                    println!("\n{}", json);
                }
            }
            Err(e) => {
                eprintln!("AI generation failed: {}", e);
                std::process::exit(1);
            }
        }
    });
}

fn fallback_demo(output: &Option<String>) {
    eprintln!("\nGenerating demo world instead...");
    let blueprint = WorldBlueprint::simple_demo();
    if let Some(ref path) = output {
        let json = serde_json::to_string_pretty(&blueprint).unwrap();
        std::fs::write(path, &json).unwrap();
        println!("Saved demo blueprint to: {}", path);
    }
}

fn inspect_blueprint(path: &str) {
    let contents = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", path, e);
            std::process::exit(1);
        }
    };

    let blueprint: WorldBlueprint = match serde_json::from_str(&contents) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Error parsing blueprint: {}", e);
            std::process::exit(1);
        }
    };

    println!("=== World Blueprint: {} ===", blueprint.name);
    println!("Description: {}", blueprint.description);
    println!("Spawn Point: {:?}", blueprint.spawn_point);
    println!();

    if !blueprint.zones.is_empty() {
        println!("--- Zones ({}) ---", blueprint.zones.len());
        for zone in &blueprint.zones {
            println!("  [{}] {}", zone.id, zone.name);
            println!("    Bounds: {:?} -> {:?}", zone.bounds.min, zone.bounds.max);
            println!("    Gravity: {:?}", zone.physics_rules.gravity);
            println!("    Air Density: {}", zone.physics_rules.air_density);
        }
        println!();
    }

    println!("--- Entities ({}) ---", blueprint.entities.len());
    for entity in &blueprint.entities {
        println!("  [{}]", entity.name);
        println!("    Mesh: {}, Position: {:?}", entity.mesh_type, entity.transform.position);
        if let Some(ref phys) = entity.physics {
            println!("    Physics: {} (mass: {}kg)", phys.body_type, phys.mass);
        }
        if !entity.tags.is_empty() {
            println!("    Tags: {:?}", entity.tags);
        }
    }
}

fn create_project(name: &str) {
    let project_dir = std::path::Path::new(name);
    if project_dir.exists() {
        eprintln!("Directory '{}' already exists", name);
        std::process::exit(1);
    }

    std::fs::create_dir_all(project_dir.join("src")).unwrap();
    std::fs::create_dir_all(project_dir.join("assets")).unwrap();

    let cargo_toml = format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2021"

[dependencies]
happen-engine = {{ path = "../" }}  # Adjust path to happen engine
"#
    );

    let main_rs = r#"use happen_engine::prelude::*;

fn main() {
    let app = HappenEngine::full();
    // Add your game systems here
    app.run();
}
"#;

    let world_json = serde_json::to_string_pretty(&WorldBlueprint::simple_demo()).unwrap();

    std::fs::write(project_dir.join("Cargo.toml"), cargo_toml).unwrap();
    std::fs::write(project_dir.join("src/main.rs"), main_rs).unwrap();
    std::fs::write(project_dir.join("assets/world.json"), world_json).unwrap();

    println!("Created new Happen project: {}", name);
    println!("  {}/src/main.rs     - Entry point", name);
    println!("  {}/assets/world.json - Demo world blueprint", name);
    println!();
    println!("To run: cd {} && cargo run", name);
}
