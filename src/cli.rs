//! Command line interface for the Lex dialogue syntax parser, converter and player.

use crate::{parse, play};
use base64::Engine;
use clap::{Parser, Subcommand};
use serde_pickle::SerOptions;

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
        #[arg(short, long)]
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

    let parse_result = parse(raw_dialogue);
    let dialogue = parse_result.dialogue;

    let duration: std::time::Duration = start.elapsed();
    println!("Parsing succeeded in: {duration:?}");

    if !parse_result.warnings.is_empty() {
        println!("\nWarnings:");
        for warning in &parse_result.warnings {
            println!("  {warning}");
        }
    }

    println!();

    match &cli.command {
        Some(Commands::Debug) => {
            println!("{dialogue:#?}");
        }

        Some(Commands::Convert { format, path }) => {
            let output = match format.as_str() {
                "json" => Some(serde_json::to_string_pretty(&dialogue).unwrap()),
                "yaml" => Some(serde_yaml::to_string(&dialogue).unwrap()),
                "ron" => Some(ron::to_string(&dialogue).unwrap()),
                "toml" => Some(toml::to_string_pretty(&dialogue).unwrap()),
                "pickle" => Some({
                    let bytes = serde_pickle::to_vec(&dialogue, SerOptions::default()).unwrap();
                    base64::prelude::BASE64_STANDARD.encode(bytes)
                }),
                _ => None,
            };

            let Some(output) = output else {
                eprintln!("Unsupported format: {format}");
                return;
            };

            let Some(path) = path else {
                println!("{output}");
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
