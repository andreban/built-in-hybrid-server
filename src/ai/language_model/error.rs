use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum AILanguageModelError {
    SystemPromptError(&'static str),
    PromptInputError(&'static str),
    ProviderError(String),
}

pub type AILanguageModelResult<T> = Result<T, AILanguageModelError>;

impl std::error::Error for AILanguageModelError {}
impl Display for AILanguageModelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AILanguageModelError::SystemPromptError(msg)
            | AILanguageModelError::PromptInputError(msg) => write!(f, "{}", msg),
            AILanguageModelError::ProviderError(msg) => write!(f, "{}", msg),
        }
    }
}

impl From<Box<dyn Error>> for AILanguageModelError {
    fn from(err: Box<dyn Error>) -> Self {
        AILanguageModelError::ProviderError(err.to_string())
    }
}
