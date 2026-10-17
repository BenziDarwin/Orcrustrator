#[cfg(feature = "cli")]
use clap::{Parser, Subcommand};
use orcrustrator::prelude::*;
#[cfg(feature = "cli")]
use std::path::Path;

#[cfg(feature = "cli")]
#[derive(Parser)]
#[command(name = "orcrustrator")]
#[command(about = "A pure, high-performance API aggregator and relational orchestrator", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[cfg(feature = "cli")]
#[derive(Subcommand)]
enum Commands {
    /// Inspect an upstream endpoint or display merged effective relational configuration
    Inspect {
        /// Base URL or full endpoint URL
        #[arg(short, long)]
        url: Option<String>,

        /// Optional path or endpoint template
        #[arg(short, long)]
        endpoint: Option<String>,

        /// Inspect the effective configuration merged from definitions.toml and overrides.toml
        #[arg(long)]
        effective: bool,

        /// Path to orcrustrator directory
        #[arg(short, long, default_value = "./orcrustrator")]
        config_dir: String,
    },
    /// Run an in-memory orchestration pipeline against an upstream API
    Orchestrate {
        /// Upstream base URL
        #[arg(short, long, default_value = "https://jsonplaceholder.typicode.com")]
        upstream: String,

        /// Resource ID to orchestrate (e.g. 1)
        #[arg(short, long, default_value = "1")]
        id: String,
    },
    /// Scan Rust source code for relation! declarations and synchronize definitions.toml
    Compile {
        /// Directory containing Rust source files
        #[arg(short, long, default_value = "./src")]
        source_dir: String,

        /// Path to the definitions manifest file
        #[arg(short, long, default_value = "./orcrustrator/definitions.toml")]
        manifest: String,

        /// Validate manifest freshness without modifying files (exits non-zero if stale)
        #[arg(long)]
        check: bool,

        /// Display unified diff between code declarations and existing manifest
        #[arg(long)]
        diff: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    #[cfg(feature = "cli")]
    {
        let cli = Cli::parse();

        match cli.command {
            Commands::Inspect { url, endpoint, effective, config_dir } => {
                if effective {
                    let base = Path::new(&config_dir);
                    let defs_path = base.join("definitions.toml");
                    let ovr_path = base.join("overrides.toml");

                    println!("Loading effective configuration from {:?}...", base);
                    let defs = DefinitionsManifest::load_from_file(&defs_path)?;
                    let ovr = OverridesManifest::load_from_file(&ovr_path)?;
                    let effective_cfg = merge_manifests(&defs, &ovr);

                    println!("\n--- Effective Relational Configuration ---");
                    println!("{}", toml::to_string_pretty(&effective_cfg).unwrap_or_default());
                    return Ok(());
                }

                let target_url = url.unwrap_or_else(|| "https://jsonplaceholder.typicode.com".to_string());
                let path = endpoint.unwrap_or_default();
                println!("Inspecting endpoint: {}{}", target_url, path);

                let client = HttpSourceClient::new(&target_url, AuthStrategy::none());
                let ep = Endpoint::get(&path);
                let params = std::collections::HashMap::new();

                let data = client.execute(&ep, &params, None).await?;
                println!("\n--- Retrieved Structured Data ---");
                println!("{}", data.to_json_string_pretty()?);

                let schema = SchemaInspector::infer(&data);
                println!("\n--- Inferred Schema Structure ---");
                println!("{}", SchemaInspector::pretty_print(&schema, 0));
            }
            Commands::Orchestrate { upstream, id } => {
                println!("Orchestrating user #{} from {}", id, upstream);

                // Build relational orchestration in memory
                let user_rel = RelationalEndpointBuilder::new("/users/{id}", "user", Endpoint::get("/users/{id}"))
                    .has_many("posts", Endpoint::get("/posts?userId={id}"), "id", "id", "posts")
                    .has_many("todos", Endpoint::get("/todos?userId={id}"), "id", "id", "todos")
                    .build();

                let service = Service::builder(upstream)
                    .register_endpoint(user_rel)
                    .build();

                let path = format!("/users/{}", id);
                let unified = service.handle_request(HttpMethod::Get, &path, None).await?;

                println!("\n--- Unified Relational Result ---");
                println!("{}", unified.to_json_string_pretty()?);
            }
            Commands::Compile { source_dir, manifest, check, diff } => {
                let src_path = Path::new(&source_dir);
                let manifest_path = Path::new(&manifest);

                println!("Scanning {:?} for relation declarations and API patterns...", src_path);
                let declared_relations = scan_source_directory(src_path)?;
                println!("Discovered {} unique relation declarations.", declared_relations.len());

                // Preserve existing manifest if 0 relations were found in code
                if declared_relations.is_empty() && manifest_path.exists() {
                    if let Ok(existing_manifest) = DefinitionsManifest::load_from_file(manifest_path) {
                        if !existing_manifest.relations.is_empty() {
                            println!("Preserving {} existing relations in {:?}.", existing_manifest.relations.len(), manifest_path);
                            validate_dag(&existing_manifest.relations)?;
                            println!("Graph validation passed: 0 circular dependencies detected.");
                            return Ok(());
                        }
                    }
                }

                // Validate graph cycle freedom
                validate_dag(&declared_relations)?;
                println!("Graph validation passed: 0 circular dependencies detected.");

                let new_manifest = DefinitionsManifest::new(declared_relations);
                let existing_manifest = DefinitionsManifest::load_from_file(manifest_path)?;

                let manifest_diff = calculate_diff(&existing_manifest, &new_manifest);

                if diff {
                    println!("\n--- Manifest Diff ---");
                    print!("{}", manifest_diff.render_diff());
                    return Ok(());
                }

                if check {
                    if !manifest_diff.is_empty() {
                        eprintln!("\nERROR: Manifest {:?} is out of sync with source code:", manifest_path);
                        eprint!("{}", manifest_diff.render_diff());
                        std::process::exit(1);
                    }
                    println!("SUCCESS: Manifest {:?} is in sync with source code.", manifest_path);
                    return Ok(());
                }

                // Normal compile: update manifest atomically
                new_manifest.save_atomic(manifest_path)?;
                println!("Successfully compiled definitions to {:?}", manifest_path);
                if !manifest_diff.is_empty() {
                    println!("\nSummary of Changes:");
                    print!("{}", manifest_diff.render_diff());
                }
            }
        }
    }

    #[cfg(not(feature = "cli"))]
    {
        println!("Orcrustrator compiled without 'cli' feature.");
    }

    Ok(())
}
