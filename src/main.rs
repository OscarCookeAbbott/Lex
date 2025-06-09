mod cli;
use clap::Parser;
use std::time::Instant;

pub mod parser;
pub use parser::*;

pub mod player;
use player::*;

fn main() {
    let cli = cli::Cli::parse();

    let raw_dialogue = std::fs::read_to_string(&cli.file).expect("Failed to read dialogue file");

    println!("\nParsing dialogue...");

    let start = Instant::now();
    let dialogue = parse(raw_dialogue).expect("Failed to parse dialogue");
    let duration: std::time::Duration = start.elapsed();
    println!("Parsing completed in: {duration:?}");

    println!();

    match &cli.command {
        Some(cli::Commands::Debug) => {
            println!("{:#?}", dialogue);
        }

        Some(cli::Commands::Convert { format, path }) => {
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

        Some(cli::Commands::Play) | None => {
            play(dialogue);
        }
    }
}
