use crate::{Dialogue, DialogueLine, DialogueSection, DialogueStep};

#[derive(Debug, Clone, PartialEq)]
struct DialogueState {
    section: DialogueSection,
    step: DialogueStep,
}

pub fn play(mut dialogue: Dialogue) {
    println!("\nPlaying dialogue:");

    println!("\nActors:");
    for (actor_id, actor) in &dialogue.actors {
        println!("{actor_id}: {actor:?}");
    }

    println!("\nVariables:");
    for (variable_name, variable_value) in &dialogue.variables {
        println!("{variable_name}: {variable_value:?}");
    }

    println!("\nFunctions:");
    for function_name in &dialogue.functions {
        println!("{function_name:?}");
    }

    let first_section = dialogue
        .sections
        .first()
        .expect("No sections found in dialogue");

    let first_step = first_section
        .steps
        .first()
        .expect("No pages found in section");

    let first_state = DialogueState {
        section: first_section.clone(),
        step: first_step.clone(),
    };

    let mut dialogue_stack = vec![first_state];

    while let Some(current_state) = dialogue_stack.pop() {
        let mut new_state = None;

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

            DialogueStep::SectionJump(section_name) => {
                let Some(new_section) = dialogue.sections.iter().find(|s| s.name == *section_name)
                else {
                    eprintln!("Section not found: {section_name}");
                    continue;
                };

                let Some(new_page) = new_section.steps.first() else {
                    eprintln!("Section has no pages: {section_name}");
                    continue;
                };

                new_state = Some(DialogueState {
                    section: new_section.clone(),
                    step: new_page.clone(),
                });
            }

            DialogueStep::SectionBounce(section_name) => {
                let Some(new_section) = dialogue.sections.iter().find(|s| s.name == *section_name)
                else {
                    eprintln!("Section not found: {section_name}");
                    continue;
                };

                let Some(new_page) = new_section.steps.first() else {
                    eprintln!("Section has no pages: {section_name}");
                    continue;
                };

                if let Some(new_step) = find_next_state(&current_state, &dialogue) {
                    dialogue_stack.push(new_step);
                }

                new_state = Some(DialogueState {
                    section: new_section.clone(),
                    step: new_page.clone(),
                });
            }

            DialogueStep::EndJump => {
                continue;
            }

            DialogueStep::TerminateJump => {
                break;
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

            DialogueStep::VariableAssign { name, value } => {
                if dialogue.variables.contains_key(name) {
                    dialogue.variables.remove(name);
                } else {
                    eprintln!("Variable assignment not pre-existing: {name}");
                }

                dialogue.variables.insert(name.clone(), value.clone());
            }
        }

        // Simulate line playback
        std::thread::sleep(std::time::Duration::from_millis(500));

        let next_state = new_state.or(find_next_state(&current_state, &dialogue));

        if let Some(new_state) = next_state {
            dialogue_stack.push(new_state);
        }
    }

    println!("Playback completed.");
}

fn find_next_state(current_state: &DialogueState, dialogue: &Dialogue) -> Option<DialogueState> {
    // Move to the next step if it exists
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

    // Move to the next section if it exists
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
