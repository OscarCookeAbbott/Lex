use std::fs::read_to_string;

pub mod parser;
pub use parser::*;

fn main() {
    let raw_dialogue = read_to_string("dialogues/demo_all.lex").unwrap();

    println!("\nParsing dialogue...");

    let dialogue = parse(raw_dialogue).unwrap();

    println!("\nDialogue:");

    println!("\nActors:");
    for (actor_id, actor) in dialogue.actors {
        println!("{actor_id}: {actor:?}");
    }

    println!("\nVariables:");
    for (variable_name, variable_value) in dialogue.variables {
        println!("{variable_name}: {variable_value:?}");
    }

    for section in dialogue.sections {
        println!("\n# {}", section.name);

        for page in section.pages {
            println!();

            for line in page.lines {
                println!("{line:?}");
            }
        }
    }
}
