use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum AILanguageModelError {
    SystemPromptError(&'static str),
    PromptInputError(&'static str),
}

impl std::error::Error for AILanguageModelError {}
impl Display for AILanguageModelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AILanguageModelError::SystemPromptError(msg)
            | AILanguageModelError::PromptInputError(msg) => write!(f, "{}", msg),
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AILanguageModelPromptRole {
    System,
    #[default]
    User,
    Assistant,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum AILanguageModelPrompt {
    Text {
        role: AILanguageModelPromptRole,
        content: String,
    },
    Image {
        role: AILanguageModelPromptRole,
        content: Vec<u8>,
    },
    Audio {
        role: AILanguageModelPromptRole,
        content: Vec<u8>,
    },
}

impl AILanguageModelPrompt {
    pub fn is_system_prompt(&self) -> bool {
        match self {
            AILanguageModelPrompt::Text { role, .. }
            | AILanguageModelPrompt::Audio { role, .. }
            | AILanguageModelPrompt::Image { role, .. } => {
                role == &AILanguageModelPromptRole::System
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum AILanguageModelExpectedInput {
    Text { languages: Vec<String> },
    Image { languages: Vec<String> },
    Audio { languages: Vec<String> },
}

///
/// See https://source.chromium.org/chromium/chromium/src/+/main:third_party/blink/renderer/modules/ai/ai_language_model_create_options.idl
///
#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AILanguageModelCreateOptions {
    pub temperature: f32,
    pub top_k: u32,
    pub expected_inputs: Vec<AILanguageModelExpectedInput>,
    pub system_prompt: Option<String>,
    pub initial_prompts: Vec<AILanguageModelPrompt>,
}

impl AILanguageModelCreateOptions {
    pub fn system_prompt_text(&self) -> Result<Option<String>, AILanguageModelError> {
        let initial_system_prompts = self
            .initial_prompts
            .iter()
            .filter(|prompt| prompt.is_system_prompt())
            .collect::<Vec<_>>();

        if self.system_prompt.is_some() {
            if !initial_system_prompts.is_empty() {
                return Err(AILanguageModelError::SystemPromptError(
                    "System prompt is already set.",
                ));
            }

            return Ok(self.system_prompt.clone());
        }

        if initial_system_prompts.len() > 1 {
            return Err(AILanguageModelError::SystemPromptError(
                "Only one system prompt is allowed.",
            ));
        }

        let Some(initial_system_prompt) = initial_system_prompts.first() else {
            return Ok(None);
        };

        let AILanguageModelPrompt::Text { content, .. } = initial_system_prompt else {
            return Err(AILanguageModelError::SystemPromptError(
                "System prompt is not a text prompt.",
            ));
        };

        Ok(Some(content.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_ai_language_model_prompt_dict() {
        let prompt = AILanguageModelPrompt::Text {
            role: AILanguageModelPromptRole::default(),
            content: "Hello, world!".to_string(),
        };

        assert_eq!(
            serde_json::to_string(&prompt).unwrap(),
            r#"{"type":"text","role":"user","content":"Hello, world!"}"#
        );
    }

    #[test]
    fn deserializes_ai_language_model_prompt_dict() {
        let prompt: AILanguageModelPrompt =
            serde_json::from_str(r#"{"type":"text","role":"user","content":"Hello, world!"}"#)
                .unwrap();

        assert_eq!(
            prompt,
            AILanguageModelPrompt::Text {
                role: AILanguageModelPromptRole::default(),
                content: "Hello, world!".to_string(),
            }
        );
    }

    #[test]
    fn serialize_ai_language_model_expected_input() {
        let input = AILanguageModelExpectedInput::Text {
            languages: vec!["en".to_string()],
        };
        assert_eq!(
            serde_json::to_string(&input).unwrap(),
            r#"{"type":"text","languages":["en"]}"#
        )
    }

    #[test]
    fn deserialize_ai_language_model_expected_input() {
        let input: AILanguageModelExpectedInput =
            serde_json::from_str(r#"{"type":"text","languages":["en"]}"#).unwrap();

        assert_eq!(
            input,
            AILanguageModelExpectedInput::Text {
                languages: vec!["en".to_string()],
            }
        );
    }
}
