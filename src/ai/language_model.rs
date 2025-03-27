use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq)]
pub enum AILanguageModelPromptRole {
    System,
    #[default]
    User,
    Assistant,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq)]
pub struct AILanguageModelPromptDict {
    pub role: AILanguageModelPromptRole,
    pub _type: AILanguageModelPromptType,
    pub content: String,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq)]
pub enum AILanguageModelPromptType {
    #[default]
    Text,
    Image,
    Audio,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq)]
pub struct AILanguageModelExpectedInput {
    pub _type: AILanguageModelPromptType,
    pub languages: Vec<String>,
}

///
/// See https://source.chromium.org/chromium/chromium/src/+/main:third_party/blink/renderer/modules/ai/ai_language_model_create_options.idl
///
#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq)]
pub struct AILanguageModelCreateOptions {
    pub temperature: f32,
    pub top_k: u32,
    pub expected_inputs: Vec<AILanguageModelExpectedInput>,
    pub system_prompt: String,
    pub initial_prompts: Vec<AILanguageModelPromptDict>,
}

pub struct AILanguageModel {
    pub create_options: AILanguageModelCreateOptions,
}

impl AILanguageModel {
    pub fn new(create_options: AILanguageModelCreateOptions) -> Self {
        Self { create_options }
    }

    pub fn prompt(input: &[AILanguageModelPromptDict]) -> String {
        serde_json::to_string(input).unwrap()
    }
}
