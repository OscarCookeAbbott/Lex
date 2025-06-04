use std::collections::HashMap;

#[derive(Default, Clone, Debug, PartialEq)]
pub struct Dialogue {
    // Metadata
    pub actors: HashMap<String, DialogueActor>,
    pub variables: HashMap<String, DialogueVariable>,
    pub functions: HashMap<String, DialogueFunction>,

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
    SectionBounce(String),
    SectionJump(String),
    EndJump,
    TerminateJump,
}

#[derive(Clone, Debug, PartialEq)]
pub enum DialogueVariable {
    Text(String),
    Number(f64),
    Boolean(bool),
    Array(Vec<String>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct DialogueFunction {
    pub args: Option<HashMap<String, DialogueVariable>>,
    pub result: Option<DialogueVariable>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DialogueActor {
    pub name: String,
    pub properties: HashMap<String, DialogueVariable>,
}
