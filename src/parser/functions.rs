use super::*;
use std::collections::*;
use std::iter::Peekable;
use std::str::Lines;
use std::string::*;

/// Result of parsing a dialogue, including any warnings encountered
#[derive(Debug)]
pub struct ParseResult {
    pub dialogue: Dialogue,
    pub warnings: Vec<String>,
}

/// Context passed to parsing functions for consistent error reporting and data access
#[derive(Debug)]
pub struct ParseContext<'a> {
    pub dialogue: &'a Dialogue,
    pub current_line: usize,
    pub warnings: &'a mut Vec<String>,
}

/// Parses the given dialogue string into a dialogue data structure.
///
/// This is the main entry point for parsing dialogue syntax. It processes the input
/// line by line, building up sections, actors, variables, and functions.
///
/// # Arguments
/// * `from` - The dialogue content as a string
///
/// # Returns
/// A `ParseResult` containing the parsed dialogue and any warnings encountered
///
/// # Example
/// ```
/// use dialogue_syntax::parser::functions::parse;
///
/// let input = r#"
/// # intro
/// Hello world!
///
/// @player
/// name: Player
///
/// player: How are you?
/// "#;
///
/// let result = parse(input.to_string());
/// assert!(!result.dialogue.sections.is_empty());
/// ```
pub fn parse(from: String) -> ParseResult {
    // Setup
    let mut dialogue = Dialogue::default();
    let mut warnings = Vec::new();

    let mut current_section = DialogueSection {
        name: META_SECTION_NAME.to_string(),
        ..Default::default()
    };

    let mut lines = from.lines().peekable();
    let mut line_number = 0;

    while let Some(line) = lines.next().map(|line| line.trim()) {
        line_number += 1;

        // Skip for empty lines
        if line.is_empty() {
            continue;
        }

        let mut context = ParseContext {
            dialogue: &dialogue,
            current_line: line_number,
            warnings: &mut warnings,
        };

        if let Some(new_section) = parse_section(line) {
            // If the current section has pages, push it to the dialogue
            if !current_section.steps.is_empty() {
                dialogue.sections.push(current_section);
            }

            // Start a new section
            current_section = new_section;

            continue;
        }

        if let Some((actor_id, actor)) = parse_actor_definition(line, &mut lines) {
            dialogue.actors.insert(actor_id, actor);
            continue;
        }

        if let Some(log_step) = parse_log_step(line) {
            current_section.steps.push(log_step);
            continue;
        }

        if let Some(comment_step) = parse_comment_step(line) {
            current_section.steps.push(comment_step);
            continue;
        }

        if let Some((function_id, function)) = parse_function_definition(line) {
            dialogue.functions.insert(function_id, function);
            continue;
        }

        if let Some((variable_name, variable_value)) = parse_variable_definition(line) {
            dialogue.variables.insert(variable_name, variable_value);
            continue;
        }

        if let Some(variable_assign_step) = parse_variable_assignment(line, &mut context) {
            current_section.steps.push(variable_assign_step);
            continue;
        }

        if let Some(section_bounce_step) = parse_section_bounce(line) {
            current_section.steps.push(section_bounce_step);
            continue;
        }

        if let Some(section_jump_step) = parse_section_jump(line) {
            current_section.steps.push(section_jump_step);
            continue;
        }

        if let Some(new_page) = parse_page(line, &mut lines, &mut context) {
            current_section.steps.push(new_page);
            continue;
        }
    }

    if !current_section.steps.is_empty() {
        dialogue.sections.push(current_section);
    }

    ParseResult { dialogue, warnings }
}

// =====================================
// Basic Element Parsing Functions
// =====================================

/// Parses a section header line.
///
/// Sections organize dialogue content into logical groups.
///
/// # Syntax
/// `# section_name`
///
/// # Example
/// ```
/// # intro
/// # main_menu
/// # ending
/// ```
fn parse_section(line: &str) -> Option<DialogueSection> {
    let section_name = line
        .strip_prefix(syntax::prefixes::SECTION)
        .map(|s| s.trim())?;

    Some(DialogueSection {
        name: section_name.to_string(),
        steps: Vec::new(),
    })
}

/// Parses log steps for debugging and development.
///
/// Log steps help with dialogue debugging by providing different severity levels.
/// These steps are typically processed by development tools but may be ignored in production.
///
/// # Syntax
/// - `/// text` - Info log (general information)
/// - `//? text` - Warning log (potential issues)
/// - `//! text` - Error log (serious problems)
///
/// # Example
/// ```
/// /// Starting dialogue system
/// //? Player name not set, using default
/// //! Critical error: save file corrupted
/// ```
fn parse_log_step(line: &str) -> Option<DialogueStep> {
    // Check for info log - `/// text`
    if let Some(log_text) = line.strip_prefix(syntax::comments::INFO) {
        return Some(DialogueStep::LogInfo(log_text.trim().to_string()));
    }

    // Check for warning log - `//? text`
    if let Some(log_text) = line.strip_prefix(syntax::comments::WARNING) {
        return Some(DialogueStep::LogWarning(log_text.trim().to_string()));
    }

    // Check for error log - `//! text`
    line.strip_prefix(syntax::comments::ERROR)
        .map(|log_text| DialogueStep::LogError(log_text.trim().to_string()))
}

/// Parses comment steps for documentation and notes.
///
/// Comments are ignored during dialogue execution but useful for documentation
/// and leaving notes for other developers or content creators.
///
/// # Syntax
/// `// text`
///
/// # Example
/// ```
/// // This section handles the introduction
/// // Remember to update voice acting scripts
/// ```
fn parse_comment_step(line: &str) -> Option<DialogueStep> {
    let comment_text = line.strip_prefix(syntax::comments::BASIC)?;
    Some(DialogueStep::Comment(comment_text.trim().to_string()))
}

// =====================================
// Variable Parsing Functions
// =====================================

/// Parses variable definitions for dialogue state management.
///
/// Variables store state that can be referenced and modified throughout the dialogue.
/// They support different data types including text, numbers, booleans, and arrays.
///
/// # Syntax
/// `$variable_name: value`
///
/// # Supported Types
/// - Text: `$name: John`
/// - Numbers: `$health: 100` or `$ratio: 3.14`
/// - Booleans: `$is_alive: true`
/// - Arrays: `$items: [sword, potion, key]`
///
/// # Example
/// ```
/// $player_name: Hero
/// $level: 1
/// $has_key: false
/// $inventory: [sword, potion]
/// ```
fn parse_variable_definition(line: &str) -> Option<(String, DialogueValue)> {
    let (variable_name, variable_value) = line
        .strip_prefix(syntax::prefixes::VARIABLE)
        .and_then(|line| line.split_once(syntax::delimiters::SEPARATOR))?;

    let variable_name = variable_name.trim().to_lowercase();
    let variable_value = parse_value(variable_value.trim());

    Some((variable_name, variable_value))
}

/// Parses variable assignments for runtime state changes.
///
/// Variable assignments modify the value of previously defined variables during
/// dialogue execution. The variable must be defined before it can be assigned.
///
/// # Syntax
/// `$variable_name = value`
///
/// # Example
/// ```
/// $health = 90
/// $current_location = forest
/// $has_talked_to_npc = true
/// ```
///
/// # Notes
/// - Generates a warning if the variable hasn't been defined
/// - Variable names are converted to lowercase for consistency
fn parse_variable_assignment(line: &str, context: &mut ParseContext) -> Option<DialogueStep> {
    let (variable_name, variable_value) = line
        .strip_prefix(syntax::prefixes::VARIABLE)
        .and_then(|line| line.split_once(syntax::delimiters::ASSIGNMENT))?;

    let variable_name = variable_name.trim();
    let variable_value = parse_value(variable_value.trim());

    if !context.dialogue.variables.contains_key(variable_name) {
        context.warnings.push(format!(
            "Line {}: Static variable definition not found [{}]",
            context.current_line, variable_name
        ));
    }

    Some(DialogueStep::VariableAssign {
        name: variable_name.to_lowercase(),
        value: variable_value,
    })
}

// =====================================
// Navigation Parsing Functions
// =====================================

/// Parses section bounce steps for returnable navigation.
///
/// Section bounces allow jumping to another section and then returning to the
/// current position afterward. This is useful for reusable dialogue segments
/// like shops, help systems, or common interactions.
///
/// # Syntax
/// `=><= section_name`
///
/// # Example
/// ```
/// Do you want to visit the shop?
/// =><= shop_menu
/// Welcome back! Anything else?
/// ```
///
/// # Flow
/// 1. Execution jumps to the target section
/// 2. Target section runs to completion
/// 3. Execution returns to the line after the bounce
fn parse_section_bounce(line: &str) -> Option<DialogueStep> {
    let jump_section = line.strip_prefix(syntax::navigation::BOUNCE)?;
    Some(DialogueStep::SectionBounce(jump_section.trim().to_string()))
}

/// Parses section jump steps for permanent navigation.
///
/// Section jumps permanently redirect execution to another section or end the dialogue.
/// Unlike bounces, there is no return to the original position.
///
/// # Syntax
/// `=> section_name` or `=> end` or `=> terminate`
///
/// # Special Targets
/// - `=> end` - Gracefully ends the dialogue
/// - `=> terminate` - Immediately terminates the dialogue
/// - `=> section_name` - Jumps to the named section
///
/// # Example
/// ```
/// Thanks for playing!
/// => credits
///
/// # game_over
/// Game Over!
/// => end
/// ```
fn parse_section_jump(line: &str) -> Option<DialogueStep> {
    let jump_section = line.strip_prefix(syntax::navigation::JUMP)?;
    let jump_section = jump_section.trim();

    let new_step = match jump_section.to_lowercase().as_str() {
        "end" => DialogueStep::EndJump,
        "terminate" => DialogueStep::TerminateJump,
        _ => DialogueStep::SectionJump(jump_section.to_string()),
    };

    Some(new_step)
}

// =====================================
// Utility Functions
// =====================================

/// Converts a string value into the appropriate DialogueValue type.
///
/// This function attempts to parse the input string as different data types
/// in order of specificity: boolean, number, array, then defaults to text.
///
/// # Type Detection Order
/// 1. **Boolean**: `true` or `false` (case-sensitive)
/// 2. **Number**: Any valid f64 (e.g., `42`, `3.14`, `-1.5`)
/// 3. **Array**: Comma-separated values in brackets `[item1, item2, item3]`
/// 4. **Text**: Everything else (default)
///
/// # Examples
/// ```
/// parse_value("true")        // -> DialogueValue::Boolean(true)
/// parse_value("42")          // -> DialogueValue::Number(42.0)
/// parse_value("3.14")        // -> DialogueValue::Number(3.14)
/// parse_value("[a, b, c]")   // -> DialogueValue::Array(vec!["a", "b", "c"])
/// parse_value("hello")       // -> DialogueValue::Text("hello")
/// parse_value("not_a_bool")  // -> DialogueValue::Text("not_a_bool")
/// ```
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
    if let Some(comma_separated_values) = from
        .strip_prefix(syntax::delimiters::ARRAY_START)
        .and_then(|s| s.strip_suffix(syntax::delimiters::ARRAY_END))
    {
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

/// Determines if a line starts a new parsing step rather than continuation text.
///
/// This function is crucial for the parser to know when a line should be treated
/// as a new dialogue element vs. continuation of the previous element.
///
/// # New Step Indicators
/// - Empty lines
/// - Comments: `//`, `///`, `//?`, `//!`
/// - Actors: `@actor_name`
/// - Sections: `# section_name`
/// - Functions: `!function_name`
/// - Variables: `$variable_name`
/// - Navigation: `=><=`, `=>`
///
/// # Example
/// ```
/// This is regular text
/// This continues the same text block
/// // This starts a new comment step
/// This would be regular text again
/// @speaker: This starts speaker dialogue
/// ```
#[inline]
fn is_new_step(line: &str) -> bool {
    if line.is_empty() {
        return true;
    }

    // Check for all the prefixes that indicate new parsing steps
    line.starts_with(syntax::comments::BASIC) ||  // Includes ///, //?, //!
        line.starts_with(syntax::prefixes::ACTOR) ||
        line.starts_with(syntax::prefixes::SECTION) ||
        line.starts_with(syntax::prefixes::FUNCTION) ||
        line.starts_with(syntax::prefixes::VARIABLE) ||
        line.starts_with(syntax::navigation::BOUNCE) ||
        line.starts_with(syntax::navigation::JUMP)
}

// =====================================
// Complex Element Parsing Functions
// =====================================

fn parse_actor_definition(
    line: &str,
    lines: &mut Peekable<Lines>,
) -> Option<(String, DialogueActor)> {
    let actor_definition = line.strip_prefix(syntax::prefixes::ACTOR)?;

    // Ensure not parsing a spoken line
    if actor_definition
        .split_once(syntax::delimiters::SEPARATOR)
        .is_some()
    {
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
        let Some((property_name, property_value_raw)) = lines
            .next()
            .and_then(|line| line.split_once(syntax::delimiters::SEPARATOR))
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

/// Parses function definitions for dialogue system integration.
///
/// Functions define external hooks that can be called from dialogue,
/// allowing integration with game systems, UI updates, or other logic.
///
/// # Syntax Options
/// - `!function_name` - Simple function with no parameters or return value
/// - `!function_name()` - Explicit empty parameter list
/// - `!function_name(param1=default1, param2=default2)` - With parameters and defaults
/// - `!function_name: return_value` - With default return value
/// - `!function_name(params): return_value` - Full syntax with params and return
///
/// # Examples
/// ```
/// !save_game
/// !play_sound(file=bell.wav, volume=0.8)
/// !get_player_name: Unknown
/// !check_inventory(item=key): false
/// ```
fn parse_function_definition(line: &str) -> Option<(String, DialogueFunction)> {
    let function_definition = line.strip_prefix(syntax::prefixes::FUNCTION)?;

    let mut function_text = function_definition;

    let mut args = None;
    let mut result = None;

    // Check for result
    if let Some((function_signature, return_value)) =
        function_text.split_once(syntax::delimiters::SEPARATOR)
    {
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
                        let mut parts = arg.splitn(2, syntax::delimiters::ASSIGNMENT);
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
    context: &mut ParseContext,
) -> Option<DialogueStep> {
    let mut page_lines = Vec::new();

    page_lines.push(parse_text_line(line, context));

    while let Some(&next_line) = lines.peek() {
        if is_new_step(next_line) {
            break;
        }

        page_lines.push(parse_text_line(lines.next().unwrap(), context));
    }

    if page_lines.is_empty() {
        return None;
    }

    Some(DialogueStep::Page(page_lines))
}

fn parse_text_line(line: &str, context: &mut ParseContext) -> DialogueLine {
    // Check for responses
    // `- response text`
    if let Some(response_text) = line.strip_prefix(syntax::prefixes::RESPONSE) {
        return DialogueLine::Response {
            text: response_text.trim().to_string(),
            pages: Vec::new(),
        };
    }

    // Check for speaker line
    // `speaker: text` or `@speaker_id: text`
    if let Some((speaker, text)) = line.split_once(syntax::delimiters::SEPARATOR) {
        let speaker = speaker.trim();
        let text = text.trim().to_string();

        if !text.is_empty() {
            let mut speaker = speaker.to_string();

            // Check if actor exists
            if let Some(speaker_id) = speaker
                .strip_prefix(syntax::prefixes::ACTOR)
                .map(|speaker_id| speaker_id.trim().to_lowercase())
            {
                if !context.dialogue.actors.contains_key(speaker_id.as_str()) {
                    context.warnings.push(format!(
                        "Line {}: Actor definition not found ({})",
                        context.current_line, speaker_id
                    ));
                }

                speaker = speaker_id;
            }

            return DialogueLine::SpeakerText { speaker, text };
        }
    }

    // Default to basic text line. Any failed parsing will be visible and thus obvious in testing.
    DialogueLine::Text(line.to_string())
}
