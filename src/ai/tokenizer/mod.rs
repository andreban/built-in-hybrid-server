use std::sync::LazyLock;

use tokenizers::Tokenizer;

const TOKENIZER_BYTES: &[u8] = include_bytes!("tokenizer.json");

pub fn get_tokenizer() -> &'static Tokenizer {
    static TOKENIZER: LazyLock<Tokenizer> =
        LazyLock::new(|| Tokenizer::from_bytes(TOKENIZER_BYTES).expect("Failed to load tokenizer"));
    &TOKENIZER
}

pub fn count_tokens(text: &str) -> tokenizers::Result<usize> {
    let tokenizer = get_tokenizer();
    let encoding = tokenizer.encode(text, true)?;
    Ok(encoding.len())
}
