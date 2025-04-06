use std::sync::Arc;

use gcp_auth::TokenProvider;
use gemini_rs::prelude::{Content, GeminiClient, GenerateContentRequest, GenerationConfig, Role};
use tokio_stream::{Stream, StreamExt};

use crate::ai::{
    language_model::{
        AILanguageModel, AILanguageModelCreateOptions, AILanguageModelError, AILanguageModelPrompt,
        AILanguageModelPromptRole, CountTokens, Prompt, PromptTreaming,
        error::AILanguageModelResult,
        types::{AILanguageModelCapabilities, AILanguageModelResponsChunk},
    },
    tokenizer,
};

const GEMINI_MODEL: &str = "gemini-2.0-flash-lite-001";

// The default capabilities for the Gemini 2.0 Flash Lite model.
const CAPABILITIES: AILanguageModelCapabilities = AILanguageModelCapabilities {
    default_temperature: 1.0,
    default_top_k: 3,
    default_top_p: 0.95,
    max_temperature: 1.0,
    max_top_k: 40,
    max_tokens: 1_048_576,
};

pub struct GeminiProvider {
    create_options: AILanguageModelCreateOptions,
    gemini_client: GeminiClient<Arc<dyn TokenProvider>>,
}

impl GeminiProvider {
    pub fn new(
        gemini_client: GeminiClient<Arc<dyn TokenProvider>>,
        options: AILanguageModelCreateOptions,
    ) -> Self {
        GeminiProvider {
            gemini_client,
            create_options: options,
        }
    }

    // Concatenate the initial prompts with the request prompts.
    fn all_inputs<'a>(
        &'a self,
        inputs: &'a [AILanguageModelPrompt],
    ) -> impl Iterator<Item = &'a AILanguageModelPrompt> {
        self.create_options.initial_prompts.iter().chain(inputs)
    }

    fn build_gemini_request(
        &self,
        inputs: &[AILanguageModelPrompt],
    ) -> AILanguageModelResult<GenerateContentRequest> {
        let generation_config = GenerationConfig::builder()
            .temperature(self.create_options.temperature)
            .top_k(self.create_options.top_k as i32)
            .build();

        let mut request_builder =
            GenerateContentRequest::builder().generation_config(generation_config);

        // Set the System Prompt.
        if let Some(system_prompt) = self.create_options.system_prompt_text()? {
            request_builder = request_builder
                .system_instruction(Content::builder().add_text_part(system_prompt).build());
        }

        // Set the User / Assistant Prompts.
        let mut contents: Vec<Content> = vec![];

        for input in self.all_inputs(inputs) {
            match input {
                AILanguageModelPrompt::Text { role, content } => {
                    let role = match role {
                        AILanguageModelPromptRole::User => Role::User,
                        AILanguageModelPromptRole::Assistant => Role::Model,
                        _ => continue, // AIlanguageModelPromptRole::System only applies for the system prompt.
                    };

                    contents.push(Content::builder().role(role).add_text_part(content).build());
                }
                _ => {
                    return Err(AILanguageModelError::PromptInputError(
                        "Unsupported input type",
                    ));
                }
            }
        }
        request_builder = request_builder.contents(contents);

        Ok(request_builder.build())
    }
}

impl AILanguageModel for GeminiProvider {
    fn create_options(&mut self, options: AILanguageModelCreateOptions) {
        self.create_options = options;
    }

    fn capabilities() -> &'static AILanguageModelCapabilities {
        &CAPABILITIES
    }
}

impl Prompt for GeminiProvider {
    async fn prompt(&self, inputs: &[AILanguageModelPrompt]) -> AILanguageModelResult<String> {
        let gemini_request = self.build_gemini_request(inputs)?;
        let gemini_response = self
            .gemini_client
            .generate_content(&gemini_request, GEMINI_MODEL)
            .await
            .map_err(|e| AILanguageModelError::ProviderError(e.to_string()))?;

        let text = gemini_response
            .candidates
            .first()
            .ok_or(gemini_rs::error::Error::NoCandidatesError)
            .map_err(|e| AILanguageModelError::ProviderError(e.to_string()))?
            .get_text()
            .unwrap_or_default();

        Ok(text)
    }
}

impl PromptTreaming for GeminiProvider {
    async fn prompt_streaming(
        &self,
        inputs: &[AILanguageModelPrompt],
    ) -> AILanguageModelResult<impl Stream<Item = AILanguageModelResult<AILanguageModelResponsChunk>>>
    {
        let gemini_request = self.build_gemini_request(inputs)?;
        let stream = self
            .gemini_client
            .generate_content_stream(&gemini_request, GEMINI_MODEL)
            .await
            .map_err(|e| AILanguageModelError::ProviderError(e.to_string()))?;

        // Transform a Gemini stream into a Stream of AILanguageModelResult<String>.
        let stream = stream.filter_map(|response| {
            let response = match response {
                Ok(response) => response,

                // Event source closed error is expected when the stream ends, so instead of
                // returning an error, we return a finished chunk without data.
                Err(gemini_rs::error::Error::EventSourceClosedError) => {
                    return Some(Ok(AILanguageModelResponsChunk {
                        text: None,
                        finished: true,
                    }));
                }
                Err(e) => {
                    return Some(Err(AILanguageModelError::ProviderError(e.to_string())));
                }
            };

            // TODO: A chunk without candidates is weird, maybe return an error here.
            let Some(candidate) = response.candidates.first() else {
                return None;
            };

            let finished = candidate.finish_reason.is_some();
            let text = candidate.get_text();

            Some(Ok(AILanguageModelResponsChunk { text, finished }))
        });
        Ok(stream)
    }
}

impl CountTokens for GeminiProvider {
    fn count_tokens(&self, inputs: &[AILanguageModelPrompt]) -> AILanguageModelResult<usize> {
        let all_inputs = self.all_inputs(inputs);
        let prompt = build_gemma_prompt(&self.create_options, all_inputs)?;
        let total_tokens = tokenizer::count_tokens(&prompt)
            .map_err(|e| AILanguageModelError::ProviderError(e.to_string()))?;
        Ok(total_tokens)
    }
}

// Formats the prompt according to theh Gemma requirements.
// See https://ai.google.dev/gemma/docs/core/prompt-structure
fn build_gemma_prompt<'a>(
    create_options: &AILanguageModelCreateOptions,
    inputs: impl Iterator<Item = &'a AILanguageModelPrompt>,
) -> AILanguageModelResult<String> {
    static START_OF_TURN: &str = "<start_of_turn>";
    static END_OF_TURN: &str = "<end_of_turn>";
    static MODEL: &str = "model";
    static USER: &str = "user";

    let mut prompt = String::new();
    let mut system_prompt = create_options.system_prompt_text()?;

    for input in inputs {
        let (user_or_model, content) = match input {
            AILanguageModelPrompt::Text { role, content } => match role {
                AILanguageModelPromptRole::User => (USER, content),
                AILanguageModelPromptRole::Assistant => (MODEL, content),
                _ => continue,
            },
            _ => continue,
        };
        prompt.push_str(START_OF_TURN);
        prompt.push_str(user_or_model);
        prompt.push_str("\n");
        if let Some(system) = system_prompt.take() {
            prompt.push_str(&system);
            prompt.push_str("\n\n");
            system_prompt = None;
        }
        prompt.push_str(&content);
        prompt.push_str(END_OF_TURN);
        prompt.push_str("\n");
    }
    Ok(prompt)
}
