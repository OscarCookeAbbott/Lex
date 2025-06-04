#[cfg(test)]
use crate::*;

#[cfg(test)]
use std::collections::HashMap;

#[test]
fn test_actor_with_properties() {
    let input = r"
@Oscar
full_name: Oscar Cooke-Abbott
age: 26";

    let expected = Dialogue {
        actors: std::collections::HashMap::from([(
            "oscar".to_string(),
            DialogueActor {
                name: "Oscar".to_string(),
                properties: std::collections::HashMap::from([
                    (
                        "full_name".to_string(),
                        DialogueVariable::Text("Oscar Cooke-Abbott".to_string()),
                    ),
                    ("age".to_string(), DialogueVariable::Number(26.0)),
                ]),
            },
        )]),
        ..Default::default()
    };

    let result = parse(input.to_string()).expect("Parse failed");

    assert_eq!(result.actors, expected.actors);
}

#[test]
fn test_variable_types() {
    let input = r"
$example_text: This is some text
$example_boolean: true
$example_number: 123.456
$example_array: [This is an entry, This is also an entry]";

    let expected = Dialogue {
        variables: std::collections::HashMap::from([
            (
                "example_text".to_string(),
                DialogueVariable::Text("This is some text".to_string()),
            ),
            (
                "example_boolean".to_string(),
                DialogueVariable::Boolean(true),
            ),
            (
                "example_number".to_string(),
                DialogueVariable::Number(123.456),
            ),
            (
                "example_array".to_string(),
                DialogueVariable::Array(vec![
                    "This is an entry".to_string(),
                    "This is also an entry".to_string(),
                ]),
            ),
        ]),
        ..Default::default()
    };

    let result = parse(input.to_string()).expect("Parse failed");

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
            DialogueVariable::Number(1.0),
        )]),
        sections: vec![DialogueSection {
            name: META_SECTION_NAME.to_string(),
            pages: vec![DialoguePage {
                lines: vec![DialogueLine::VariableAssign {
                    name: "foo".to_string(),
                    value: DialogueVariable::Number(2.0),
                }],
            }],
        }],
        ..Default::default()
    };

    let result = parse(input.to_string()).expect("Parse failed");

    assert_eq!(result.variables, expected.variables);
    assert_eq!(result.sections, expected.sections);
}

#[test]
fn test_function_lines() {
    let input = r"
!example_function
!example_function_args(arg_1=Some value, arg_2=123.456)
!example_function_text: Default return value";

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
                "example_function_args".to_string(),
                DialogueFunction {
                    args: Some(HashMap::from([
                        (
                            "arg_1".to_string(),
                            DialogueVariable::Text("Some value".to_string()),
                        ),
                        ("arg_2".to_string(), DialogueVariable::Number(123.456)),
                    ])),
                    result: None,
                },
            ),
            (
                "example_function_text".to_string(),
                DialogueFunction {
                    args: None,
                    result: Some(DialogueVariable::Text("Default return value".to_string())),
                },
            ),
        ]),
        ..Default::default()
    };

    let result = parse(input.to_string()).expect("Parse failed");

    assert_eq!(result.functions, expected.functions);
}

#[test]
fn test_sections() {
    let input = r"
#Intro
Hello
#Outro
Goodbye";

    let expected = Dialogue {
        sections: vec![
            DialogueSection {
                name: "Intro".to_string(),
                pages: vec![DialoguePage {
                    lines: vec![DialogueLine::Text("Hello".to_string())],
                }],
            },
            DialogueSection {
                name: "Outro".to_string(),
                pages: vec![DialoguePage {
                    lines: vec![DialogueLine::Text("Goodbye".to_string())],
                }],
            },
        ],
        ..Default::default()
    };

    let result = parse(input.to_string()).expect("Parse failed");

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
            pages: vec![DialoguePage {
                lines: vec![
                    DialogueLine::Comment("Comment".to_string()),
                    DialogueLine::LogInfo("Info".to_string()),
                    DialogueLine::LogWarning("Warning".to_string()),
                    DialogueLine::LogError("Error".to_string()),
                ],
            }],
        }],
        ..Default::default()
    };

    let result = parse(input.to_string()).expect("Parse failed");

    assert_eq!(result, expected);
}

#[test]
fn test_speaker_lines() {
    let input = r"
@Oscar: Hello
Other Oscar: Hi";

    let expected = Dialogue {
        sections: vec![DialogueSection {
            name: META_SECTION_NAME.to_string(),
            pages: vec![DialoguePage {
                lines: vec![
                    DialogueLine::SpeakerText {
                        speaker: "oscar".to_string(),
                        text: "Hello".to_string(),
                    },
                    DialogueLine::SpeakerText {
                        speaker: "Other Oscar".to_string(),
                        text: "Hi".to_string(),
                    },
                ],
            }],
        }],
        ..Default::default()
    };

    let result = parse(input.to_string()).expect("Parse failed");

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
            pages: vec![DialoguePage {
                lines: vec![
                    DialogueLine::Response {
                        text: "Response 1".to_string(),
                        pages: vec![],
                    },
                    DialogueLine::Response {
                        text: "Nested Response".to_string(),
                        pages: vec![],
                    },
                ],
            }],
        }],
        ..Default::default()
    };

    let result = parse(input.to_string()).expect("Parse failed");

    assert_eq!(result.sections, expected.sections);
}

#[test]
fn test_manual_page_extension() {
    let input = r"
| This is a single page
|
| - Wow!
| - More!";

    let expected = Dialogue {
        sections: vec![DialogueSection {
            name: META_SECTION_NAME.to_string(),
            pages: vec![DialoguePage {
                lines: vec![
                    DialogueLine::Text("This is a single page".to_string()),
                    DialogueLine::Response {
                        text: "Wow!".to_string(),
                        pages: vec![],
                    },
                    DialogueLine::Response {
                        text: "More!".to_string(),
                        pages: vec![],
                    },
                ],
            }],
        }],
        ..Default::default()
    };

    let result = parse(input.to_string()).expect("Parse failed");

    assert_eq!(result, expected);
}

#[test]
fn test_annotated_dialogue() {
    let input = r"
[mood=info]
This is annotated.";

    let expected = Dialogue::default();

    let result = parse(input.to_string()).expect("Parse failed");

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

    let result = parse(input.to_string()).expect("Parse failed");

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

    let result = parse(input.to_string()).expect("Parse failed");

    assert_eq!(result, expected);
}

#[test]
fn test_jumps() {
    let input = r"
=> #Outro
=><= #Outro
=> END
=> TERMINATE";

    let expected = Dialogue {
        sections: vec![DialogueSection {
            name: META_SECTION_NAME.to_string(),
            pages: vec![DialoguePage {
                lines: vec![
                    DialogueLine::SectionJump("#Outro".to_string()),
                    DialogueLine::SectionBounce("#Outro".to_string()),
                    DialogueLine::EndJump,
                    DialogueLine::TerminateJump,
                ],
            }],
        }],
        ..Default::default()
    };

    let result = parse(input.to_string()).expect("Parse failed");

    assert_eq!(result, expected);
}

#[test]
fn test_variable_and_property_references() {
    let input = "This is a variable: {$var}. This is a property: {@actor.prop}.";

    let expected = Dialogue::default();

    let result = parse(input.to_string()).expect("Parse failed");

    assert_eq!(result, expected);
}

#[test]
fn test_rich_text() {
    let input = "This is **bold** and *italic*";

    let expected = Dialogue::default();

    let result = parse(input.to_string()).expect("Parse failed");

    assert_eq!(result, expected);
}
