use super::*;
use std::collections::*;
use std::string::*;

/// Parses the given dialogue string into a dialogue data structure, if possible.
pub fn parse(from: String) -> Result<Dialogue, ParseError> {
    // Setup
    let mut dialogue = Dialogue::default();

    let mut current_section = DialogueSection {
        name: META_SECTION_NAME.to_string(),
        ..Default::default()
    };

    let mut current_page: DialoguePage = DialoguePage {
        lines: Vec::with_capacity(8),
    };

    let mut lines = from.lines().peekable();

    while let Some(line) = lines.next() {
        let line = line.trim();

        // Create new page on empty line if current not empty
        if line.trim().is_empty() {
            commit_page(&mut current_section, &mut current_page);

            continue;
        }

        // Check for new section
        if let Some(section_name) = line.strip_prefix('#') {
            let section_name = section_name.trim_start();
            commit_page(&mut current_section, &mut current_page);

            // If the current section has pages, push it to the dialogue
            if !current_section.pages.is_empty() {
                dialogue.sections.push(current_section);
            }

            // Start a new section
            current_section = DialogueSection {
                name: section_name.to_string(),
                pages: Vec::new(),
            };

            continue;
        }

        // Check for actor definition
        if let Some(actor_definition) = line.strip_prefix('@') {
            // Ensure not parsing a spoken line
            if actor_definition.split_once(':').is_none() {
                let actor_name = actor_definition.trim_start();
                let mut properties = HashMap::new();

                // Sub-iterate over subsequent lines
                while let Some(&next_line) = lines.peek() {
                    if next_line.trim().is_empty()
                        || next_line.starts_with('@')
                        || next_line.starts_with('#')
                    {
                        break;
                    }

                    // Parse the next line as a property, skip sub-iteration if not possible
                    let Some((property_name, property_value_raw)) =
                        lines.next().and_then(|line| line.split_once(':'))
                    else {
                        break;
                    };
                    let property_name = property_name.trim();
                    let property_value = parse_variable_value(property_value_raw.trim());
                    properties.insert(property_name.to_lowercase(), property_value);
                }

                dialogue.actors.insert(
                    actor_name.to_lowercase(),
                    DialogueActor {
                        name: actor_name.to_string(),
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

#[inline]
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

    // Check for functions
    // `!function_name` or `!function_name()` or `!function_name(arg1, ...)` or `!function_name: {default_return_value}` etc
    if let Some(function_definition) = line.strip_prefix('!') {
        let mut function_text = function_definition;

        let mut function_args = None;
        let mut function_result = None;

        // Check for return value
        if let Some((function_signature, return_value)) = function_text.split_once(':') {
            let return_value = return_value.trim();

            function_result = Some(parse_variable_value(return_value));

            function_text = function_signature;
        }

        // Check for arguments
        if let Some((function_name, arg_definitions)) = function_text.split_once('(') {
            let args = arg_definitions.trim_end_matches(')').trim();

            // Parse arguments if any
            function_args = if !args.is_empty() {
                Some(
                    args.split(',')
                        .map(|arg| {
                            let mut parts = arg.splitn(2, '=');
                            let name = parts.next().unwrap().trim().to_string();
                            let value = parts.next().unwrap_or("").trim();

                            (name, parse_variable_value(value))
                        })
                        .collect(),
                )
            } else {
                None
            };

            function_text = function_name;
        }

        let function_name = function_text.trim().to_lowercase();

        let new_function = DialogueFunction {
            args: function_args,
            result: function_result,
        };

        dialogue.functions.insert(function_name, new_function);

        return None;
    }

    // Check for variable
    // `$variable_name: value` or `$variable_name = value`
    if let Some(line) = line.strip_prefix('$') {
        // Check for variable definition
        // `$variable_name: value`
        if let Some((name, value)) = line.split_once(':') {
            let variable_name = name.trim();
            let variable_value = value.trim();

            dialogue.variables.insert(
                variable_name.to_lowercase(),
                parse_variable_value(variable_value),
            );

            return None;
        }

        // Check for variable assign
        // `$variable_name = value`
        if let Some((name_and_op, value)) = line.split_once('=') {
            let variable_name = name_and_op.trim();
            let variable_value = value.trim();
            if !dialogue
                .variables
                .contains_key(&variable_name.to_lowercase())
            {
                eprintln!("WARNING: Static variable definition not found [{variable_name}]");
            }

            return Some(DialogueLine::VariableAssign {
                name: variable_name.to_lowercase(),
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

    // Check for section jump
    // `=><= jump_section`
    if let Some(jump_section) = line.strip_prefix("=><=") {
        return Some(DialogueLine::SectionBounce(jump_section.trim().to_string()));
    }

    // Check for section jump
    // `=> jump_section`
    if let Some(jump_section) = line.strip_prefix("=>") {
        let jump_section = jump_section.trim();
        match jump_section.to_lowercase().as_str() {
            "end" => return Some(DialogueLine::EndJump),
            "terminate" => return Some(DialogueLine::TerminateJump),
            _ => return Some(DialogueLine::SectionJump(jump_section.to_string())),
        }
    }

    // Check for speaker line
    // `speaker_id: text` or `@spaker_id: text`
    if let Some((speaker, text)) = line.split_once(':') {
        let speaker = speaker.trim();
        let text = text.trim();

        // Check if anonymous
        let Some(speaker_id) = speaker.strip_prefix('@') else {
            return Some(DialogueLine::SpeakerText {
                speaker: speaker.to_string(),
                text: text.to_string(),
            });
        };

        let speaker_id = speaker_id.trim();

        if !dialogue.actors.contains_key(&speaker_id.to_lowercase()) {
            eprintln!("WARNING: Actor definition not found ({})", speaker_id);
        }

        return Some(DialogueLine::SpeakerText {
            speaker: speaker_id.to_lowercase(),
            text: text.to_string(),
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

#[inline]
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
