//! Player module for interactive dialogue playback.

use crate::{Dialogue, DialogueLine, DialogueSection, DialogueStep};

#[derive(Debug, Clone, PartialEq)]
struct DialogueState {
    section: DialogueSection,
    step: DialogueStep,
}

/// Plays back a dialogue interactively using basic CLI.
pub fn play(mut dialogue: Dialogue) {
    let first_section = dialogue
        .sections
        .first()
        .expect("No sections found in dialogue");

    let first_step = first_section
        .steps
        .first()
        .expect("No pages found in first section");

    let first_state = DialogueState {
        section: first_section.clone(),
        step: first_step.clone(),
    };

    let mut dialogue_stack = vec![first_state];

    while let Some(current_state) = dialogue_stack.pop() {
        let mut next_state = None;

        match &current_state.step {
            DialogueStep::Comment(_) => {}

            DialogueStep::LogInfo(text) => {
                println!("{text}");
            }

            DialogueStep::LogWarning(text) => {
                println!("{text}");
            }

            DialogueStep::LogError(text) => {
                eprintln!("{text}");
            }

            DialogueStep::VariableAssign { name, value } => {
                if dialogue.variables.contains_key(name) {
                    dialogue.variables.remove(name);
                } else {
                    eprintln!("Variable assignment not pre-existing: {name}");
                }

                dialogue.variables.insert(name.clone(), value.clone());
            }

            DialogueStep::Page(lines) => {
                let output_lines = lines.iter().map(|line| match line {
                    DialogueLine::Text(text) => text.clone(),

                    DialogueLine::SpeakerText { speaker, text } => {
                        let speaker_name = dialogue
                            .actors
                            .get(speaker)
                            .map_or(speaker, |actor| &actor.name);

                        format!("{speaker_name}: {text}")
                    }

                    DialogueLine::Response { text, pages: _ } => {
                        format!("- {text}")
                    }
                });

                let output = output_lines.collect::<Vec<_>>().join("\n");

                println!("{output}");
            }

            DialogueStep::SectionJump(section_name) => {
                next_state = get_section_state_by_name(section_name, &dialogue);
            }

            DialogueStep::SectionBounce(section_name) => {
                next_state = get_section_state_by_name(section_name, &dialogue);

                if next_state.is_some() {
                    if let Some(next_step) = get_next_step_state(&current_state, &dialogue) {
                        dialogue_stack.push(next_step);
                    }
                }
            }

            DialogueStep::EndJump => {
                continue;
            }

            DialogueStep::TerminateJump => {
                break;
            }
        }

        // Simulate line playback
        std::thread::sleep(std::time::Duration::from_millis(500));

        if let Some(next_state) = next_state.or_else(|| get_next_step_state(&current_state, &dialogue)) {
            dialogue_stack.push(next_state);
        }
    }

    println!("Playback completed.");
}

/// Gets the next sequential step in the dialogue as a state, if it exists.
fn get_next_step_state(
    current_state: &DialogueState,
    dialogue: &Dialogue,
) -> Option<DialogueState> {
    // Move to the current section's next step if it exists
    if let Some(next_step) = current_state
        .section
        .steps
        .iter()
        .skip_while(|&l| *l != current_state.step)
        .nth(1)
    {
        return Some(DialogueState {
            section: current_state.section.clone(),
            step: next_step.clone(),
        });
    }

    // Move to the subsequent section if it exists
    if let Some(next_section) = dialogue
        .sections
        .iter()
        .skip_while(|&s| *s != current_state.section)
        .nth(1)
    {
        return Some(DialogueState {
            section: next_section.clone(),
            step: next_section.steps.first().unwrap().clone(),
        });
    }

    None
}

fn get_section_state_by_name(section_name: &str, dialogue: &Dialogue) -> Option<DialogueState> {
    let Some(new_section) = dialogue.sections.iter().find(|s| s.name == *section_name) else {
        eprintln!("Section not found: {section_name}");
        return None;
    };

    let Some(new_page) = new_section.steps.first() else {
        eprintln!("Section has no pages: {section_name}");
        return None;
    };

    Some(DialogueState {
        section: new_section.clone(),
        step: new_page.clone(),
    })
}
