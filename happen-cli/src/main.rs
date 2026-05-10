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
    /// Run a demo scene (or list available demos)
    Demo {
        /// Demo name (e.g. plaza, playground). Omit to list all.
        name: Option<String>,
    },

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
        Commands::Demo { name } => run_demo(name),
        Commands::Blueprint { file } => run_blueprint(&file),
        Commands::Generate { prompt, output, provider, model } => {
            generate_world(&prompt, output, &provider, model)
        }
        Commands::Inspect { file } => inspect_blueprint(&file),
        Commands::New { name } => create_project(&name),
    }
}

fn run_demo(name: Option<String>) {
    let demos_dir = find_demos_dir();

    let name = match name {
        Some(n) => n,
        None => {
            list_demos(&demos_dir);
            return;
        }
    };

    let path = demos_dir.join(format!("{}.json", name));
    if !path.exists() {
        eprintln!("Demo '{}' not found.", name);
        eprintln!();
        list_demos(&demos_dir);
        std::process::exit(1);
    }

    run_blueprint(path.to_str().unwrap());
}

fn find_demos_dir() -> std::path::PathBuf {
    let candidates = [
        std::path::PathBuf::from("demos"),
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.join("../../demos")))
            .unwrap_or_default(),
    ];
    for c in &candidates {
        if c.is_dir() {
            return c.clone();
        }
    }
    std::path::PathBuf::from("demos")
}

fn list_demos(demos_dir: &std::path::Path) {
    println!("Happen Engine v{}", env!("CARGO_PKG_VERSION"));
    println!();

    if !demos_dir.is_dir() {
        println!("No demos/ directory found.");
        println!("Create demos/ with .json blueprint files to add demos.");
        return;
    }

    let mut demos: Vec<(String, String, usize)> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(demos_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "json").unwrap_or(false) {
                let name = path.file_stem().unwrap().to_string_lossy().to_string();
                if let Ok(contents) = std::fs::read_to_string(&path) {
                    if let Ok(bp) = serde_json::from_str::<WorldBlueprint>(&contents) {
                        demos.push((name, bp.description, bp.entities.len()));
                    }
                }
            }
        }
    }

    demos.sort_by(|a, b| a.0.cmp(&b.0));

    if demos.is_empty() {
        println!("No demos found in {}", demos_dir.display());
        println!("Add .json blueprint files to create demos.");
    } else {
        println!("Available demos:");
        println!();
        for (name, desc, count) in &demos {
            println!("  {:<16} {} ({} entities)", name, desc, count);
        }
        println!();
        println!("Run with: happen demo <name>");
        println!("Example:  happen demo plaza");
    }
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

    println!("Happen Engine - '{}'", blueprint.name);
    println!("  {}", blueprint.description);
    println!("  Zones: {}  Entities: {}", blueprint.zones.len(), blueprint.entities.len());
    println!();
    println!("  Controls:");
    println!("    Click     - Lock mouse / enable controls");
    println!("    WASD      - Move");
    println!("    Mouse     - Look around");
    println!("    Space     - Jump");
    println!("    Shift     - Sprint");
    println!("    Escape    - Release mouse");
    println!();

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
