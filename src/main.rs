use std::fs::read_to_string;
use std::time::Instant;

pub mod parser;
pub use parser::*;

fn main() {
    let raw_dialogue = read_to_string("dialogues/demo_all.lex").unwrap();

    println!("\nParsing dialogue...");

    let start = Instant::now();
    let dialogue = parse(raw_dialogue).unwrap();
    let duration = start.elapsed();
    println!("Parsing completed in: {duration:?}");

    println!("\nDialogue:");

    println!("\nActors:");
    for (actor_id, actor) in dialogue.actors {
        println!("{actor_id}: {actor:?}");
    }

    println!("\nVariables:");
    for (variable_name, variable_value) in dialogue.variables {
        println!("{variable_name}: {variable_value:?}");
    }

    println!("\nFunctions:");
    for function_name in dialogue.functions {
        println!("{function_name:?}");
    }

    for section in dialogue.sections {
        println!();

        for page in section.pages {
            println!();

            for line in page.lines {
                println!("{line:?}");
            }
        }
    }
}
