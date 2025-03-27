use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct PromptOptions {
    pub temperature: f32,
    pub top_k: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Role {
    User,
    Assistant,
    System,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Prompt {
    pub role: Role,
    pub content: String,
}
