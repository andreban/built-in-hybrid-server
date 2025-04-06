mod error;
pub mod providers;
mod types;

pub use error::AILanguageModelError;
use error::AILanguageModelResult;
use tokio_stream::Stream;
use types::AILanguageModelCapabilities;
pub use types::AILanguageModelCreateOptions;
pub use types::AILanguageModelPrompt;
pub use types::AILanguageModelPromptRole;

pub trait AILanguageModel {
    fn create_options(&mut self, options: AILanguageModelCreateOptions);
    fn capabilities() -> &'static AILanguageModelCapabilities;
}

pub trait Prompt: AILanguageModel {
    fn prompt(
        &self,
        inputs: &[AILanguageModelPrompt],
    ) -> impl Future<Output = AILanguageModelResult<String>>;
}

pub trait PromptTreaming: AILanguageModel {
    fn prompt_streaming(
        &self,
        inputs: &[AILanguageModelPrompt],
    ) -> impl Future<Output = AILanguageModelResult<impl Stream<Item = AILanguageModelResult<String>>>>;
}

pub trait CountTokens: AILanguageModel {
    fn count_tokens(&self, inputs: &[AILanguageModelPrompt]) -> AILanguageModelResult<usize>;
}
