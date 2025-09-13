#[cfg(test)]
use crate::*;

#[cfg(test)]
use std::collections::HashMap;

/// Helper function for tests - extracts dialogue from ParseResult and panics on warnings
fn parse_test_helper(input: &str) -> super::types::Dialogue {
    let result = super::functions::parse(input.to_string());
    if !result.warnings.is_empty() {
        panic!("Unexpected warnings: {:?}", result.warnings);
    }
    result.dialogue
}

#[test]
fn test_actor_definitions() {
    let input = r"
@Oscar
name: Oscar Robin
age: 26";

    let expected = Dialogue {
        actors: std::collections::HashMap::from([(
            "oscar".to_string(),
            DialogueActor {
                name: "Oscar Robin".to_string(),
                properties: std::collections::HashMap::from([
                    (
                        "name".to_string(),
                        DialogueValue::Text("Oscar Robin".to_string()),
                    ),
                    ("age".to_string(), DialogueValue::Number(26.0)),
                ]),
            },
        )]),
        ..Default::default()
    };

    let result = parse_test_helper(input);

    assert_eq!(result.actors, expected.actors);
}

#[test]
fn test_variable_definitions() {
    let input = r"
$example_text: This is some text
$example_boolean: true
$example_number: 123.456
$example_array: [This is an entry, This is also an entry]";

    let expected = Dialogue {
        variables: std::collections::HashMap::from([
            (
                "example_text".to_string(),
                DialogueValue::Text("This is some text".to_string()),
            ),
            ("example_boolean".to_string(), DialogueValue::Boolean(true)),
            ("example_number".to_string(), DialogueValue::Number(123.456)),
            (
                "example_array".to_string(),
                DialogueValue::Array(vec![
                    "This is an entry".to_string(),
                    "This is also an entry".to_string(),
                ]),
            ),
        ]),
        ..Default::default()
    };

    let result = parse_test_helper(input);

    assert_eq!(result.variables, expected.variables);
}

#[test]
fn test_variable_assignment() {
    let input = r"
$foo: 1
$foo = 2";

    let expected = Dialogue {
        variables: std::collections::HashMap::from([(
            "foo".to_string(),
            DialogueValue::Number(1.0),
        )]),
        sections: vec![DialogueSection {
            name: META_SECTION_NAME.to_string(),
            steps: vec![DialogueStep::VariableAssign {
                name: "foo".to_string(),
                value: DialogueValue::Number(2.0),
            }],
        }],

        ..Default::default()
    };

    let result = parse_test_helper(input);

    assert_eq!(result.variables, expected.variables);
    assert_eq!(result.sections, expected.sections);
}

#[test]
fn test_function_definitions() {
    let input = r"
!example_function
!example_function_result: Default return value
!example_function_args(arg_1=Some value, arg_2=123.456)
!example_function_args_result(arg_1=Some value, arg_2=123.456): Default return value";

    let expected = Dialogue {
        functions: HashMap::from([
            (
                "example_function".to_string(),
                DialogueFunction {
                    args: None,
                    result: None,
                },
            ),
            (
                "example_function_result".to_string(),
                DialogueFunction {
                    args: None,
                    result: Some(DialogueValue::Text("Default return value".to_string())),
                },
            ),
            (
                "example_function_args".to_string(),
                DialogueFunction {
                    args: Some(HashMap::from([
                        (
                            "arg_1".to_string(),
                            DialogueValue::Text("Some value".to_string()),
                        ),
                        ("arg_2".to_string(), DialogueValue::Number(123.456)),
                    ])),
                    result: None,
                },
            ),
            (
                "example_function_args_result".to_string(),
                DialogueFunction {
                    args: Some(HashMap::from([
                        (
                            "arg_1".to_string(),
                            DialogueValue::Text("Some value".to_string()),
                        ),
                        ("arg_2".to_string(), DialogueValue::Number(123.456)),
                    ])),
                    result: Some(DialogueValue::Text("Default return value".to_string())),
                },
            ),
        ]),
        ..Default::default()
    };

    let result = parse_test_helper(input);

    assert_eq!(result.functions, expected.functions);
}

#[test]
fn test_section_definitons() {
    let input = r"
#Intro
Hello
#Outro
Goodbye";

    let expected = Dialogue {
        sections: vec![
            DialogueSection {
                name: "Intro".to_string(),
                steps: vec![DialogueStep::Page(vec![DialogueLine::Text(
                    "Hello".to_string(),
                )])],
            },
            DialogueSection {
                name: "Outro".to_string(),
                steps: vec![DialogueStep::Page(vec![DialogueLine::Text(
                    "Goodbye".to_string(),
                )])],
            },
        ],
        ..Default::default()
    };

    let result = parse_test_helper(input);

    assert_eq!(result.sections, expected.sections);
}

#[test]
fn test_comments_and_logs() {
    let input = r"
// Comment
/// Info
//? Warning
//! Error";

    let expected = Dialogue {
        sections: vec![DialogueSection {
            name: META_SECTION_NAME.to_string(),
            steps: vec![
                DialogueStep::Comment("Comment".to_string()),
                DialogueStep::LogInfo("Info".to_string()),
                DialogueStep::LogWarning("Warning".to_string()),
                DialogueStep::LogError("Error".to_string()),
            ],
        }],
        ..Default::default()
    };

    let result = parse_test_helper(input);

    assert_eq!(result, expected);
}

#[test]
fn test_spoken_dialogue() {
    let input = r"
@Oscar
name: Oscar Robin

@Oscar: Hello
Other Oscar: Hi";

    let expected = Dialogue {
        actors: HashMap::from([(
            "oscar".to_string(),
            DialogueActor {
                name: "Oscar Robin".to_string(),
                properties: std::collections::HashMap::from([(
                    "name".to_string(),
                    DialogueValue::Text("Oscar Robin".to_string()),
                )]),
            },
        )]),
        sections: vec![DialogueSection {
            name: META_SECTION_NAME.to_string(),
            steps: vec![DialogueStep::Page(vec![
                DialogueLine::SpeakerText {
                    speaker: "oscar".to_string(),
                    text: "Hello".to_string(),
                },
                DialogueLine::SpeakerText {
                    speaker: "Other Oscar".to_string(),
                    text: "Hi".to_string(),
                },
            ])],
        }],
        ..Default::default()
    };

    let result = parse_test_helper(input);

    assert_eq!(result, expected);
}

#[test]
fn test_responses_and_nesting() {
    let input = r"
- Response 1
    - Nested Response";

    let expected = Dialogue {
        sections: vec![DialogueSection {
            name: META_SECTION_NAME.to_string(),
            steps: vec![DialogueStep::Page(vec![
                DialogueLine::Response {
                    text: "Response 1".to_string(),
                    pages: vec![],
                },
                DialogueLine::Response {
                    text: "Nested Response".to_string(),
                    pages: vec![],
                },
            ])],
        }],
        ..Default::default()
    };

    let result = parse_test_helper(input);

    assert_eq!(result.sections, expected.sections);
}

#[test]
fn test_manual_page_extensions() {
    let input = r"
| This is a single page
|
| - Wow!
| - More!";

    let expected = Dialogue {
        sections: vec![DialogueSection {
            name: META_SECTION_NAME.to_string(),
            steps: vec![DialogueStep::Page(vec![
                DialogueLine::Text("This is a single page".to_string()),
                DialogueLine::Response {
                    text: "Wow!".to_string(),
                    pages: vec![],
                },
                DialogueLine::Response {
                    text: "More!".to_string(),
                    pages: vec![],
                },
            ])],
        }],
        ..Default::default()
    };

    let result = parse_test_helper(input);

    assert_eq!(result, expected);
}

#[test]
fn test_annotated_dialogue() {
    let input = r"
[mood=info]
This is annotated.";

    let expected = Dialogue::default();

    let result = parse_test_helper(input);

    assert_eq!(result, expected);
}

#[test]
fn test_conditional_blocks() {
    let input = r"
[if=$var]
Text
~ ELSE
Other
~";

    let expected = Dialogue::default();

    let result = parse_test_helper(input);

    assert_eq!(result, expected);
}

#[test]
fn test_branching_repeat_while_each() {
    let input = r"
~ REPEAT 3
Repeat
~
~ WHILE $var < 10
While
~
~ EACH $arr as $item
Each
~";

    let expected = Dialogue::default();

    let result = parse_test_helper(input);

    assert_eq!(result, expected);
}

#[test]
fn test_jumps_and_bounces() {
    let input = r"
=> #Outro
=><= #Outro
=> END
=> TERMINATE";

    let expected = Dialogue {
        sections: vec![DialogueSection {
            name: META_SECTION_NAME.to_string(),
            steps: vec![
                DialogueStep::SectionJump("#Outro".to_string()),
                DialogueStep::SectionBounce("#Outro".to_string()),
                DialogueStep::EndJump,
                DialogueStep::TerminateJump,
            ],
        }],
        ..Default::default()
    };

    let result = parse_test_helper(input);

    assert_eq!(result, expected);
}

#[test]
fn test_data_references() {
    let input = "This is a variable: {$var}. This is a property: {@actor.prop}.";

    let expected = Dialogue::default();

    let result = parse_test_helper(input);

    assert_eq!(result, expected);
}

#[test]
fn test_rich_text() {
    let input = "This is **bold** and *italic*";

    let expected = Dialogue::default();

    let result = parse_test_helper(input);

    assert_eq!(result, expected);
}
