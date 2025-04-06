#[test]
pub fn test_prompt_text() {
    use built_in_hybrid_server::ai::language_model::AILanguageModelPrompt;
    use built_in_hybrid_server::ai::language_model::AILanguageModelPromptRole;

    let prompt = AILanguageModelPrompt::Text {
        role: AILanguageModelPromptRole::default(),
        content: "Hello, world!".to_string(),
    };

    assert_eq!(
        serde_json::to_string(&prompt).unwrap(),
        r#"{"type":"text","role":"user","content":"Hello, world!"}"#
    );
}
