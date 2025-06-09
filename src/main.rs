mod cli;

pub mod parser;
pub use parser::*;

pub mod player;
pub use player::*;

fn main() {
    cli::execute();
}
