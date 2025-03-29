# Built-in AI API Hybrid Server

## Prompt API
 - [Explainer](https://github.com/webmachinelearning/prompt-api) 

### Feature support status
 - createOptions
    - [x] `systemPrompt`
    - [x] `initialPrompts`
    - [x] `temperature`
    - [x] `topK`
    - [ ] `expectedInputs`
 - LanguageModel
    - [x] `model.prompt()`.
    - [x] `model.promptStreaming()`.
    - [ ] `model.countPromptTokens()`.
    - [ ] `model.tokensLeft` / `model.tokensSoFar`
    - [ ] `model.maxTokens`
    - [x] `model.maxTemperature`
    - [x] `model.maxTopK`
    - [x] `model.defaultTemperature`
    - [x] `model.defaultTopK`
