use clap::{Parser, Subcommand};
use happen_engine::prelude::*;

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
        Commands::Generate { prompt, output } => generate_world(&prompt, output),
        Commands::Inspect { file } => inspect_blueprint(&file),
        Commands::New { name } => create_project(&name),
    }
}

fn run_demo() {
    println!("Happen Engine v{}", env!("CARGO_PKG_VERSION"));
    println!("Starting demo scene...");

    let blueprint = WorldBlueprint::simple_demo();
    let app = HappenEngine::from_blueprint(blueprint);
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

fn generate_world(prompt: &str, output: Option<String>) {
    println!("Happen Engine - AI World Generator");
    println!("Prompt: \"{}\"", prompt);

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let config = happen_ai::AiProviderConfig::default();
        let provider = match happen_ai::AnthropicProvider::from_config(&config) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Error: {}", e);
                eprintln!("Set ANTHROPIC_API_KEY environment variable to use AI generation.");
                eprintln!("\nGenerating demo world instead...");

                let blueprint = WorldBlueprint::simple_demo();
                if let Some(ref path) = output {
                    let json = serde_json::to_string_pretty(&blueprint).unwrap();
                    std::fs::write(path, &json).unwrap();
                    println!("Saved demo blueprint to: {}", path);
                }
                return;
            }
        };

        let orchestrator = happen_ai::AiOrchestrator::new(Box::new(provider));
        let intent = happen_ai::UserIntent::new(prompt);

        println!("Generating world with AI...");

        match orchestrator.generate_world(&intent).await {
            Ok(blueprint) => {
                println!("Generated: '{}'", blueprint.name);
                println!("  {} zones, {} entities", blueprint.zones.len(), blueprint.entities.len());

                if let Some(ref path) = output {
                    let json = serde_json::to_string_pretty(&blueprint).unwrap();
                    std::fs::write(path, &json).unwrap();
                    println!("Saved to: {}", path);
                } else {
                    let json = serde_json::to_string_pretty(&blueprint).unwrap();
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
