use std::{collections::HashMap, string::ParseError};

mod tests;

// STRUCTS
#[derive(Default, Clone, Debug, PartialEq)]
pub struct Dialogue {
    // Metadata
    pub actors: HashMap<String, DialogueActor>,
    pub variables: HashMap<String, DialogueVariable>,
    pub functions: Vec<String>,

    // Content
    pub sections: Vec<DialogueSection>,
}

#[derive(Default, Clone, Debug, PartialEq)]
pub struct DialogueSection {
    pub name: String,
    pub pages: Vec<DialoguePage>,
}

#[derive(Default, Clone, Debug, PartialEq)]
pub struct DialoguePage {
    pub lines: Vec<DialogueLine>,
}

/// Describes the content of a given line of dialogue.
#[derive(Clone, Debug, PartialEq)]
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

#[derive(Clone, Debug, PartialEq)]
pub enum DialogueVariable {
    Text(String),
    Number(f64),
    Boolean(bool),
    Array(Vec<String>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct DialogueActor {
    pub name: String,
    pub properties: HashMap<String, DialogueVariable>,
}

// FUNCTIONS
/// Parses the given dialogue string into a dialogue data structure, if possible.
pub fn parse(from: String) -> Result<Dialogue, ParseError> {
    // Setup
    let mut dialogue = Dialogue::default();

    let mut current_section = DialogueSection {
        name: "Meta".to_string(),
        ..Default::default()
    };

    let mut current_page: DialoguePage = DialoguePage {
        lines: Vec::with_capacity(8),
    };

    let mut lines = from.lines().peekable();

    while let Some(line) = lines.next() {
        // Create new page on empty line if current not empty
        if line.trim().is_empty() {
            commit_page(&mut current_section, &mut current_page);

            continue;
        }

        // Check for new section
        if let Some(section_name) = line.strip_prefix('#') {
            let section_name = section_name.trim().to_string();

            commit_page(&mut current_section, &mut current_page);

            // If the current section has pages, push it to the dialogue
            if !current_section.pages.is_empty() {
                dialogue.sections.push(current_section);
            }

            // Start a new section
            current_section = DialogueSection {
                name: section_name,
                pages: Vec::new(),
            };

            continue;
        }

        // Check for actor definition
        if let Some(actor_definition) = line.strip_prefix('@') {
            // Ensure not parsing a spoken line
            if actor_definition.split_once(':').is_none() {
                let actor_name = actor_definition.trim().to_string();
                let mut properties = HashMap::new();

                // Sub-iterate over subsequent lines
                while let Some(&next_line) = lines.peek() {
                    // End actor definition parsing on new pages
                    if next_line.trim().is_empty()
                        || next_line.strip_prefix('@').is_some()
                        || next_line.strip_prefix('#').is_some()
                    {
                        break;
                    }

                    // Parse the next line as a property, skip sub-iteration if not possible
                    let Some((property_name, property_value_raw)) =
                        lines.next().and_then(|line| line.split_once(':'))
                    else {
                        break;
                    };

                    let property_name = property_name.trim().to_lowercase();
                    let property_value = parse_variable_value(property_value_raw.trim());

                    properties.insert(property_name, property_value);
                }

                dialogue.actors.insert(
                    actor_name.to_lowercase(),
                    DialogueActor {
                        name: actor_name,
                        properties,
                    },
                );

                continue;
            }
        }

        // Otherwise parse by line
        if let Some(new_line) = parse_line(line, &mut dialogue) {
            current_page.lines.push(new_line);
        }
    }

    commit_page(&mut current_section, &mut current_page);

    dialogue.sections.push(current_section);

    Ok(dialogue)
}

fn parse_line(line: &str, dialogue: &mut Dialogue) -> Option<DialogueLine> {
    let mut line = line.trim();

    // Clean any manual page extensions
    // `| text` or `|text`
    if let Some(page_line) = line.strip_prefix('|') {
        if page_line.is_empty() {
            return None;
        }

        line = page_line.trim();
    }

    // Check for info log
    // `/// text`
    if let Some(log_text) = line.strip_prefix("///") {
        return Some(DialogueLine::LogInfo(log_text.trim().to_string()));
    }

    // Check for warning log
    // `//? text`
    if let Some(log_text) = line.strip_prefix("//?") {
        return Some(DialogueLine::LogWarning(log_text.trim().to_string()));
    }

    // Check for error log
    // `//! text`
    if let Some(log_text) = line.strip_prefix("//!") {
        return Some(DialogueLine::LogError(log_text.trim().to_string()));
    }

    // Check for basic comment
    // `// text`
    if let Some(comment_text) = line.strip_prefix("//") {
        return Some(DialogueLine::Comment(comment_text.trim().to_string()));
    }

    // Check for speaker line
    // `speaker_id: text`
    if let Some((speaker_id, text)) = line.strip_prefix('@').and_then(|line| line.split_once(':')) {
        let speaker_id = speaker_id.trim().to_lowercase();
        let text = text.trim();

        if !dialogue.actors.contains_key(speaker_id.as_str()) {
            println!("WARNING: Actor definition not found ({})", speaker_id)
        }

        return Some(DialogueLine::SpeakerText {
            speaker: speaker_id,
            text: text.to_string(),
        });
    }

    // Check for functions
    // `!function_name` or `!function_name()` or `!function_name(arg1, ...)` or `!function_name: {default_return_value}` etc
    if let Some(function_name) = line.strip_prefix('!') {
        let function_text = function_name.trim().to_string();

        dialogue.functions.push(function_text);

        return None;
    }

    // Check for variable
    // `$variable_name: value` or `$variable_name = value`
    if let Some(line) = line.strip_prefix('$') {
        // Check for variable definition
        // `$variable_name: value`
        if let Some((name, value)) = line.split_once(':') {
            let variable_name = name.trim().to_lowercase();
            let variable_value = value.trim();

            dialogue.variables.insert(
                variable_name.trim().to_string(),
                parse_variable_value(variable_value),
            );

            return None;
        }

        // Check for variable assign
        // `$variable_name = value`
        if let Some((name_and_op, value)) = line.split_once('=') {
            let variable_name = name_and_op.trim().to_lowercase();
            let variable_value = value.trim();

            if !dialogue.variables.contains_key(&variable_name) {
                println!("WARNING: Static variable definition not found [{variable_name}]");
            }

            return Some(DialogueLine::VariableAssign {
                name: variable_name,
                value: parse_variable_value(variable_value),
            });
        }
    }

    // Check for responses
    // `- response text`
    if let Some(response_text) = line.strip_prefix('-') {
        return Some(DialogueLine::Response {
            text: response_text.trim().to_string(),
            pages: Vec::new(),
        });
    }

    // Default to basic text line. Any failed parsing will be visible and thus obvious in testing.
    Some(DialogueLine::Text(line.to_string()))
}

/// Converts the given string into a `DialogueVariable`, as a boolean, number, array or otherwise text.
fn parse_variable_value(from: &str) -> DialogueVariable {
    // Check for boolean value
    if let Ok(bool_value) = str::parse::<bool>(from) {
        return DialogueVariable::Boolean(bool_value);
    }

    // Check for numeric value
    if let Ok(number_value) = str::parse::<f64>(from) {
        return DialogueVariable::Number(number_value);
    }

    // Check for array of unquoted strings
    // `[first item, second item, third item]`
    if let Some(comma_separated_values) = from.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
        let values: Vec<String> = comma_separated_values
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        return DialogueVariable::Array(values);
    }

    // Default to textual value
    DialogueVariable::Text(from.to_string())
}

fn commit_page(current_section: &mut DialogueSection, current_page: &mut DialoguePage) {
    if !current_page.lines.is_empty() {
        current_section.pages.push(std::mem::replace(
            current_page,
            DialoguePage {
                lines: Vec::with_capacity(8),
            },
        ));
    }
}
