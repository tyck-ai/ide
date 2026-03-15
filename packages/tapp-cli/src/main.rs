mod commands;
mod templates;

use clap::{Parser, Subcommand};
use colored::Colorize;

#[derive(Parser)]
#[command(name = "tapp")]
#[command(about = "CLI tool for building Tapp extensions for Tyck IDE")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Output JSON instead of human-readable text
    #[arg(long, global = true)]
    json: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new Tapp app
    Init {
        /// Name of the app
        name: String,

        /// Template to use
        #[arg(short, long, default_value = "minimal")]
        template: String,
    },

    /// Build the app for development (watch mode)
    Dev {
        /// Path to the app (defaults to current directory)
        #[arg(default_value = ".")]
        path: String,
    },

    /// Build the app for release
    Build {
        /// Path to the app (defaults to current directory)
        #[arg(default_value = ".")]
        path: String,

        /// Build in release mode
        #[arg(long, default_value = "true")]
        release: bool,
    },

    /// Install an app
    Install {
        /// Path to the app directory or .tapp file
        path: String,
    },

    /// List installed apps
    List,

    /// Uninstall an app
    Uninstall {
        /// App ID to uninstall
        id: String,
    },

    /// Run an installed app
    Run {
        /// App ID to run
        id: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Init { name, template } => commands::init::run(&name, &template, cli.json).await,
        Commands::Dev { path } => commands::dev::run(&path, cli.json).await,
        Commands::Build { path, release } => commands::build::run(&path, release, cli.json).await,
        Commands::Install { path } => commands::install::run(&path, cli.json).await,
        Commands::List => commands::list::run(cli.json).await,
        Commands::Uninstall { id } => commands::uninstall::run(&id, cli.json).await,
        Commands::Run { id } => commands::run::run(&id, cli.json).await,
    };

    if let Err(e) = result {
        if cli.json {
            let error = serde_json::json!({
                "success": false,
                "error": e.to_string()
            });
            println!("{}", serde_json::to_string_pretty(&error)?);
        } else {
            eprintln!("{} {}", "Error:".red().bold(), e);
        }
        std::process::exit(1);
    }

    Ok(())
}
