use clap::{Parser, Subcommand};

/// Dialogue Syntax CLI
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Path to the dialogue file
    #[arg(short, long)]
    pub file: String,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Parse the dialogue file and display the raw parsed data
    Debug,

    /// Play the parsed dialogue interactively
    Play,

    /// Convert the parsed dialogue to a specific format
    Convert {
        /// Output format (json, yaml, etc)
        #[arg(short, long, default_value = "json")]
        format: String,

        /// Output file path (default: stdout)
        path: Option<String>,
    },
}
