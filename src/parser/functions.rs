use super::*;
use std::collections::*;
use std::iter::Peekable;
use std::str::Lines;
use std::string::*;

/// Parses the given dialogue string into a dialogue data structure, if possible.
pub fn parse(from: String) -> Result<Dialogue, ParseError> {
    // Setup
    let mut dialogue = Dialogue::default();

    let mut current_section = DialogueSection {
        name: META_SECTION_NAME.to_string(),
        ..Default::default()
    };

    let mut lines = from.lines().peekable();

    while let Some(line) = lines.next().map(|line| line.trim()) {
        // Skip for empty lines
        if line.is_empty() {
            continue;
        }

        // Check for new section
        if let Some(section_name) = line.strip_prefix('#').map(|s| s.trim()) {
            // If the current section has pages, push it to the dialogue
            if !current_section.steps.is_empty() {
                dialogue.sections.push(current_section);
            }

            // Start a new section
            current_section = DialogueSection {
                name: section_name.to_string(),
                steps: Vec::new(),
            };

            continue;
        }

        // Actor definition
        // `@actor_name
        // property_name: value
        // ...`
        if let Some((actor_id, actor)) = parse_actor_definition(line, &mut lines) {
            dialogue.actors.insert(actor_id, actor);
            continue;
        }

        // Check for info log
        // `/// text`
        if let Some(log_text) = line.strip_prefix("///") {
            let new_step = DialogueStep::LogInfo(log_text.trim().to_string());

            current_section.steps.push(new_step);

            continue;
        }

        // Check for warning log
        // `//? text`
        if let Some(log_text) = line.strip_prefix("//?") {
            let new_step = DialogueStep::LogWarning(log_text.trim().to_string());

            current_section.steps.push(new_step);

            continue;
        }

        // Check for error log
        // `//! text`
        if let Some(log_text) = line.strip_prefix("//!") {
            let new_step = DialogueStep::LogError(log_text.trim().to_string());

            current_section.steps.push(new_step);

            continue;
        }

        // Basic comment
        // `// text`
        if let Some(comment_text) = line.strip_prefix("//") {
            let new_step = DialogueStep::Comment(comment_text.trim().to_string());

            current_section.steps.push(new_step);

            continue;
        }

        // Function definition
        // `!function_name` or `!function_name()` or `!function_name(arg1, ...)` or `!function_name: {default_return_value}` etc
        if let Some((function_id, function)) = parse_function_definition(line) {
            dialogue.functions.insert(function_id, function);
            continue;
        }

        // Variable definition
        // `$variable_name: value`
        if let Some((variable_name, variable_value)) =
            line.strip_prefix('$').and_then(|line| line.split_once(':'))
        {
            let variable_name = variable_name.trim().to_lowercase();
            let variable_value = parse_value(variable_value.trim());

            dialogue.variables.insert(variable_name, variable_value);

            continue;
        }

        // Variable assignment
        // `$variable_name = value`
        if let Some((variable_name, variable_value)) =
            line.strip_prefix('$').and_then(|line| line.split_once('='))
        {
            let variable_name = variable_name.trim();
            let variable_value = parse_value(variable_value.trim());

            if !dialogue.variables.contains_key(variable_name) {
                eprintln!("WARNING: Static variable definition not found [{variable_name}]");
            }

            let new_step = DialogueStep::VariableAssign {
                name: variable_name.to_lowercase(),
                value: variable_value,
            };

            current_section.steps.push(new_step);

            continue;
        }

        // Section bounce
        // `=><= jump_section`
        if let Some(jump_section) = line.strip_prefix("=><=") {
            let new_step = DialogueStep::SectionBounce(jump_section.trim().to_string());

            current_section.steps.push(new_step);

            continue;
        }

        // Section jump
        // `=> jump_section`
        if let Some(jump_section) = line.strip_prefix("=>") {
            let jump_section = jump_section.trim();

            let new_step = match jump_section.to_lowercase().as_str() {
                "end" => DialogueStep::EndJump,
                "terminate" => DialogueStep::TerminateJump,
                _ => DialogueStep::SectionJump(jump_section.to_string()),
            };

            current_section.steps.push(new_step);

            continue;
        }

        // Default to display text parsing
        if let Some(new_page) = parse_page(line, &mut lines, &dialogue) {
            current_section.steps.push(new_page);
            continue;
        }
    }

    if !current_section.steps.is_empty() {
        dialogue.sections.push(current_section);
    }

    Ok(dialogue)
}

/// Converts the given string into a dialogue value type.
fn parse_value(from: &str) -> DialogueValue {
    // Check for boolean value
    if let Ok(bool_value) = str::parse::<bool>(from) {
        return DialogueValue::Boolean(bool_value);
    }

    // Check for numeric value
    if let Ok(number_value) = str::parse::<f64>(from) {
        return DialogueValue::Number(number_value);
    }

    // Check for array of unquoted strings
    // `[first item, second item, third item]`
    if let Some(comma_separated_values) = from.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
        let values: Vec<String> = comma_separated_values
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        return DialogueValue::Array(values);
    }

    // Default to textual value
    DialogueValue::Text(from.to_string())
}

#[inline]
fn is_new_step(line: &str) -> bool {
    let checks = ["//", "///", "//!", "//?", "@", "#", "!", "$", "=><=", "=>"];

    if line.is_empty() {
        return true;
    }

    checks.iter().any(|&check| line.starts_with(check))
}

fn parse_actor_definition(
    line: &str,
    lines: &mut Peekable<Lines>,
) -> Option<(String, DialogueActor)> {
    let actor_definition = line.strip_prefix('@')?;

    // Ensure not parsing a spoken line
    if actor_definition.split_once(':').is_some() {
        return None;
    }

    let actor_name = actor_definition.trim();
    let mut properties = HashMap::new();

    // Sub-iterate over subsequent lines
    while let Some(&next_line) = lines.peek() {
        // If we hit a new step, break out of the sub-iteration
        if is_new_step(next_line) {
            break;
        }

        // Parse the next line as a property, cancel sub-iteration if not possible
        let Some((property_name, property_value_raw)) =
            lines.next().and_then(|line| line.split_once(':'))
        else {
            break;
        };

        let property_name = property_name.trim().to_lowercase();
        let property_value = parse_value(property_value_raw.trim());

        properties.insert(property_name, property_value);
    }

    let display_name = match properties.get("name") {
        Some(DialogueValue::Text(override_name)) => override_name.as_str(),
        _ => actor_name,
    };

    Some((
        actor_name.to_lowercase(),
        DialogueActor {
            name: display_name.to_string(),
            properties,
        },
    ))
}

fn parse_function_definition(line: &str) -> Option<(String, DialogueFunction)> {
    let function_definition = line.strip_prefix('!')?;

    let mut function_text = function_definition;

    let mut args = None;
    let mut result = None;

    // Check for result
    if let Some((function_signature, return_value)) = function_text.split_once(':') {
        let return_value = return_value.trim();

        result = Some(parse_value(return_value));

        function_text = function_signature;
    }

    // Check for arguments
    if let Some((function_name, arg_definitions)) = function_text.split_once('(') {
        let arg_definitions = arg_definitions.trim_end_matches(')').trim();

        // Parse arguments if any
        args = if !arg_definitions.is_empty() {
            Some(
                arg_definitions
                    .split(',')
                    .map(|arg| {
                        let mut parts = arg.splitn(2, '=');
                        let name = parts.next().unwrap().trim().to_string();
                        let value = parts.next().unwrap_or("").trim();

                        (name, parse_value(value))
                    })
                    .collect(),
            )
        } else {
            None
        };

        function_text = function_name;
    }

    let function_name = function_text.trim().to_lowercase();

    let new_function = DialogueFunction { args, result };

    Some((function_name, new_function))
}

fn parse_page(
    line: &str,
    lines: &mut Peekable<Lines>,
    dialogue: &Dialogue,
) -> Option<DialogueStep> {
    let mut page_lines = Vec::new();

    page_lines.push(parse_text_line(line, dialogue));

    while let Some(&next_line) = lines.peek() {
        if is_new_step(next_line) {
            break;
        }

        page_lines.push(parse_text_line(lines.next().unwrap(), dialogue));
    }

    if page_lines.is_empty() {
        return None;
    }

    Some(DialogueStep::Page(page_lines))
}

fn parse_text_line(line: &str, dialogue: &Dialogue) -> DialogueLine {
    // Check for responses
    // `- response text`
    if let Some(response_text) = line.strip_prefix('-') {
        return DialogueLine::Response {
            text: response_text.trim().to_string(),
            pages: Vec::new(),
        };
    }

    // Check for speaker line
    // `speaker: text` or `@speaker_id: text`
    if let Some((speaker, text)) = line.split_once(':') {
        let speaker = speaker.trim();
        let text = text.trim().to_string();

        if !text.is_empty() {
            let mut speaker = speaker.to_string();

            // Check if actor exists
            if let Some(speaker_id) = speaker
                .strip_prefix('@')
                .map(|speaker_id| speaker_id.trim().to_lowercase())
            {
                if !dialogue.actors.contains_key(speaker_id.as_str()) {
                    eprintln!("WARNING: Actor definition not found ({})", speaker_id);
                }

                speaker = speaker_id;
            }

            return DialogueLine::SpeakerText { speaker, text };
        }
    }

    // Default to basic text line. Any failed parsing will be visible and thus obvious in testing.
    DialogueLine::Text(line.to_string())
}
