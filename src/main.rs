use std::fs::read_to_string;
use std::time::Instant;

pub mod parser;
pub use parser::*;

pub mod player;
use player::*;

fn main() {
    let raw_dialogue = read_to_string("dialogues/demo_all.lex").unwrap();

    println!("\nParsing dialogue...");

    let start = Instant::now();
    let dialogue = parse(raw_dialogue).unwrap();
    let duration: std::time::Duration = start.elapsed();
    println!("Parsing completed in: {duration:?}");

    play(dialogue);
}
