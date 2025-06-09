use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Dialogue {
    // Metadata
    pub actors: HashMap<String, DialogueActor>,
    pub variables: HashMap<String, DialogueValue>,
    pub functions: HashMap<String, DialogueFunction>,

    // Content
    pub sections: Vec<DialogueSection>,
}

#[derive(Default, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DialogueSection {
    pub name: String,
    pub steps: Vec<DialogueStep>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DialogueStep {
    Comment(String),
    LogInfo(String),
    LogWarning(String),
    LogError(String),
    Page(Vec<DialogueLine>),
    VariableAssign {
        name: String,
        value: DialogueValue,
    },
    SectionBounce(String),
    SectionJump(String),
    EndJump,
    TerminateJump,
}

/// Describes the content of a given line of dialogue.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DialogueLine {
    Text(String),
    SpeakerText {
        speaker: String,
        text: String,
    },
    Response {
        text: String,
        pages: Vec<DialogueStep>,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DialogueValue {
    Text(String),
    Number(f64),
    Boolean(bool),
    Array(Vec<String>),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DialogueFunction {
    pub args: Option<HashMap<String, DialogueValue>>,
    pub result: Option<DialogueValue>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DialogueActor {
    pub name: String,
    pub properties: HashMap<String, DialogueValue>,
}
