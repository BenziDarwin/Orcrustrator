#[cfg(feature = "cli")]
use clap::{Parser, Subcommand};
use orcrustrator::prelude::*;

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
    /// Inspect an upstream endpoint, detect its format, and infer its data schema
    Inspect {
        /// Base URL or full endpoint URL
        #[arg(short, long)]
        url: String,

        /// Optional path or endpoint template
        #[arg(short, long)]
        endpoint: Option<String>,
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
}

#[tokio::main]
async fn main() -> Result<()> {
    #[cfg(feature = "cli")]
    {
        let cli = Cli::parse();

        match cli.command {
            Commands::Inspect { url, endpoint } => {
                let path = endpoint.unwrap_or_default();
                println!("Inspecting endpoint: {}{}", url, path);

                let client = HttpSourceClient::new(&url, AuthStrategy::none());
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
        }
    }

    #[cfg(not(feature = "cli"))]
    {
        println!("Orcrustrator compiled without 'cli' feature.");
    }

    Ok(())
}
