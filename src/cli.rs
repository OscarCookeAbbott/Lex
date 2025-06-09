//! Command line interface for the Lex dialogue syntax parser, converter and player.

use crate::{parse, play};
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

pub fn execute() {
    let cli = Cli::parse();

    let raw_dialogue = std::fs::read_to_string(&cli.file).expect("Failed to read file");

    println!("\nParsing dialogue...");

    let start = std::time::Instant::now();

    let dialogue = parse(raw_dialogue).expect("Parsing failed");

    let duration: std::time::Duration = start.elapsed();
    println!("Parsing succeeded in: {duration:?}");

    println!();

    match &cli.command {
        Some(Commands::Debug) => {
            println!("{:#?}", dialogue);
        }

        Some(Commands::Convert { format, path }) => {
            let output = match format.as_str() {
                "json" => Some(serde_json::to_string_pretty(&dialogue).unwrap().to_string()),
                _ => None,
            };

            let Some(output) = output else {
                eprintln!("Unsupported format: {format}");
                return;
            };

            let Some(path) = path else {
                println!("{}", output);
                return;
            };

            std::fs::write(path, output).expect("Failed to write output file");

            println!("Output written to: {path}");
        }

        Some(Commands::Play) | None => {
            play(dialogue);
        }
    }
}
