use std::{collections::HashMap, string::ParseError};

#[derive(Default, Clone, Debug)]
pub struct Dialogue {
    // Metadata
    pub actors: HashMap<String, DialogueActor>,
    pub variables: HashMap<String, DialogueVariable>,
    pub functions: Vec<String>,

    // Content
    pub sections: Vec<DialogueSection>,
}

#[derive(Default, Clone, Debug)]
pub struct DialogueSection {
    pub name: String,
    pub pages: Vec<DialoguePage>,
}

#[derive(Default, Clone, Debug)]
pub struct DialoguePage {
    pub lines: Vec<DialogueLine>,
}

/// Describes the content of a given line of dialogue.
#[derive(Clone, Debug)]
pub enum DialogueLine {
    Comment(String),
    LogInfo(String),
    LogWarning(String),
    LogError(String),
    Text(String),
    SpeakerText {
        speaker: String,
        text: String,
    },
    Response {
        text: String,
        pages: Vec<DialoguePage>,
    },
    VariableAssign {
        name: String,
        value: DialogueVariable,
    },
}

#[derive(Clone, Debug)]
pub enum DialogueVariable {
    Text(String),
    Number(f64),
    Boolean(bool),
    Array(Vec<String>),
}

#[derive(Clone, Debug)]
pub struct DialogueActor {
    pub name: String,
    pub properties: HashMap<String, DialogueVariable>,
}

/// Parses the given dialogue string into a dialogue data structure, if possible.
pub fn parse(from: String) -> Result<Dialogue, ParseError> {
    let mut dialogue = Dialogue::default();

    let dialogue_lines = from.split('\n');

    let mut current_section = DialogueSection {
        name: "Meta".to_string(),
        ..Default::default()
    };

    let mut current_page = DialoguePage::default();

    for mut line in dialogue_lines {
        // Check for new section
        if let Some(line_text) = line.strip_prefix('#') {
            dialogue.sections.push(current_section);

            current_section = DialogueSection {
                name: line_text.trim().to_string(),
                ..Default::default()
            };

            continue;
        }

        // Check for new page
        if line.is_empty() {
            if !current_page.lines.is_empty() {
                current_section.pages.push(current_page);

                current_page = DialoguePage::default();
            }

            continue;
        }

        // Check for manual page extensions
        if let Some(page_line) = line.trim().strip_prefix('|') {
            if page_line.is_empty() {
                continue;
            }

            line = page_line;
        }

        // Check for comment
        if let Some(comment_text) = line.strip_prefix("//") {
            // Check for info log
            if let Some(log_text) = comment_text.strip_prefix("/") {
                current_page
                    .lines
                    .push(DialogueLine::LogInfo(log_text.trim().to_string()));

                continue;
            }

            // Check for warning log
            if let Some(log_text) = comment_text.strip_prefix("?") {
                current_page
                    .lines
                    .push(DialogueLine::LogWarning(log_text.trim().to_string()));

                continue;
            }

            // Check for error log
            if let Some(log_text) = comment_text.strip_prefix("!") {
                current_page
                    .lines
                    .push(DialogueLine::LogError(log_text.trim().to_string()));

                continue;
            }

            // Otherwise basic comment
            current_page
                .lines
                .push(DialogueLine::Comment(comment_text.trim().to_string()));

            continue;
        }

        // Check for actor
        if let Some(actor_text) = line.strip_prefix('@') {
            if let Some((speaker_id, text)) = line.split_once(':') {
                if !dialogue
                    .actors
                    .contains_key(speaker_id.trim().to_lowercase().as_str())
                {
                    println!(
                        "WARNING: Actor definition not found ({})",
                        speaker_id.trim().to_lowercase()
                    )
                }

                current_page.lines.push(DialogueLine::SpeakerText {
                    speaker: speaker_id.trim().to_lowercase().to_string(),
                    text: text.trim().to_string(),
                });

                continue;
            }

            // Default actor definition
            dialogue.actors.insert(
                actor_text.trim().to_lowercase().to_string(),
                DialogueActor {
                    name: actor_text.trim().to_string(),
                    properties: HashMap::new(),
                },
            );

            continue;
        }

        // Check for variable
        if let Some(variable_expression) = line.strip_prefix('$') {
            // Check for definition
            if let Some((variable_name, variable_value)) = variable_expression.split_once(":") {
                // Check for boolean value
                if let Ok(bool_value) = str::parse::<bool>(variable_value.trim()) {
                    dialogue.variables.insert(
                        variable_name.trim().to_string(),
                        DialogueVariable::Boolean(bool_value),
                    );
                    continue;
                }

                // Check for numeric value
                if let Ok(number_value) = str::parse::<f64>(variable_value.trim()) {
                    dialogue.variables.insert(
                        variable_name.trim().to_string(),
                        DialogueVariable::Number(number_value),
                    );
                    continue;
                }

                // TODO: Add array parsing

                // Default to textual value
                dialogue.variables.insert(
                    variable_name.trim().to_string(),
                    DialogueVariable::Text(variable_value.trim().to_string()),
                );
                continue;
            }

            // Check for assignment
            if let Some((variable_name, variable_value)) = variable_expression.split_once("=") {
                // Check for boolean value
                if let Ok(bool_value) = str::parse::<bool>(variable_value.trim()) {
                    if !dialogue.variables.contains_key(variable_name.trim()) {
                        println!(
                            "WARNING: Variable definition not found [{}]",
                            variable_name.trim()
                        );
                    }

                    current_page.lines.push(DialogueLine::VariableAssign {
                        name: variable_name.trim().to_string(),
                        value: DialogueVariable::Boolean(bool_value),
                    });
                    continue;
                }

                // Check for numeric value
                if let Ok(number_value) = str::parse::<f64>(variable_value.trim()) {
                    if !dialogue.variables.contains_key(variable_name.trim()) {
                        println!(
                            "WARNING: Variable definition not found [{}]",
                            variable_name.trim()
                        );
                    }

                    current_page.lines.push(DialogueLine::VariableAssign {
                        name: variable_name.trim().to_string(),
                        value: DialogueVariable::Number(number_value),
                    });
                    continue;
                }

                // TODO: Add array parsing

                // Default to textual value
                if !dialogue.variables.contains_key(variable_name.trim()) {
                    println!(
                        "WARNING: Variable definition not found [{}]",
                        variable_name.trim()
                    );
                }

                current_page.lines.push(DialogueLine::VariableAssign {
                    name: variable_name.trim().to_string(),
                    value: DialogueVariable::Text(variable_value.trim().to_string()),
                });
                continue;
            }
        }

        // Check for responses
        if let Some(response_text) = line.trim().strip_prefix('-') {
            let new_response = DialogueLine::Response {
                text: response_text.trim().to_string(),
                pages: vec![],
            };

            current_page.lines.push(new_response);

            continue;
        }

        // Default to basic text line
        current_page
            .lines
            .push(DialogueLine::Text(line.to_string()));
    }

    current_section.pages.push(current_page);
    dialogue.sections.push(current_section);

    Ok(dialogue)
}
