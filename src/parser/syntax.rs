//! Dialogue syntax constants organized by functionality.
//! 
//! This module contains all the string literals used to identify different
//! dialogue syntax elements during parsing.

/// Element prefixes that start new dialogue components
pub mod prefixes {
    /// Section header prefix: `# section_name`
    pub const SECTION: &str = "#";
    
    /// Actor definition prefix: `@actor_name`
    pub const ACTOR: &str = "@";
    
    /// Function definition prefix: `!function_name`
    pub const FUNCTION: &str = "!";
    
    /// Variable definition/assignment prefix: `$variable_name`
    pub const VARIABLE: &str = "$";
    
    /// Response option prefix: `- response text`
    pub const RESPONSE: &str = "-";
}

/// Comment and logging prefixes
pub mod comments {
    /// Basic comment prefix: `// comment text`
    pub const BASIC: &str = "//";
    
    /// Info log prefix: `/// info message`
    pub const INFO: &str = "///";
    
    /// Warning log prefix: `//? warning message`
    pub const WARNING: &str = "//?";
    
    /// Error log prefix: `//! error message`
    pub const ERROR: &str = "//!";
}

/// Navigation control prefixes
pub mod navigation {
    /// Section bounce prefix (call and return): `=><= section_name`
    pub const BOUNCE: &str = "=><=";
    
    /// Section jump prefix (permanent redirect): `=> section_name`
    pub const JUMP: &str = "=>";
}

/// Separators and delimiters used in parsing
pub mod delimiters {
    /// Variable separator for key-value pairs: `key: value`
    pub const SEPARATOR: &str = ":";
    
    /// Variable assignment operator: `$var = value`
    pub const ASSIGNMENT: &str = "=";
    
    /// Array opening bracket: `[item1, item2]`
    pub const ARRAY_START: &str = "[";
    
    /// Array closing bracket: `[item1, item2]`
    pub const ARRAY_END: &str = "]";
}
