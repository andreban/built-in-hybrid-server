use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
pub struct SummarizerOptions {
    #[serde(default)]
    pub shared_context: String,
    pub summary_type: SummaryType,
    pub summary_format: SummaryFormat,
    pub summary_length: SummaryLength,
}

#[derive(Debug, Default, Deserialize, PartialEq)]
pub enum SummaryType {
    #[serde(rename = "headline")]
    Headline,

    #[serde(rename = "tl;dr")]
    #[default]
    TLDR,

    #[serde(rename = "key-points")]
    KeyPoints,

    #[serde(rename = "teaser")]
    Teaser,
}

#[derive(Debug, Default, Deserialize, PartialEq)]
pub enum SummaryFormat {
    #[default]
    PlainText,
    Markdown,
}

#[derive(Debug, Default, Deserialize, PartialEq)]
pub enum SummaryLength {
    Short,
    #[default]
    Medium,
    Long,
}
pub fn build_system_prompt(input: &str, options: &SummarizerOptions) -> String {
    let mut system_prompt = String::new();
    system_prompt.push_str(match options.summary_type {
        SummaryType::Headline => {
            "You are a skilled copy editor crafting headlines to capture attention and convey the essence of the content provided in the 'TEXT' section."
        }
        _ =>  "You are a skilled assistant that accurately summarizes content provided in the 'TEXT' section.",
    });

    system_prompt.push_str(match options.summary_type {
        SummaryType::TLDR => {
            "Summarize the text as if explaining it to someone with a very short attention span.\n"
        }
        SummaryType::KeyPoints => {
            "Extract the main points of the text and present them as a bulleted list.\n"
        }
        SummaryType::Teaser => {
            "Craft an enticing summary that encourages the user to read the full text.\n"
        }
        SummaryType::Headline => {
            "Generate a headline that effectively summarizes the main point of the text.\n"
        }
    });

    system_prompt.push_str(
        match (
            &options.summary_length,
            &options.summary_type,
        ) {
            (SummaryLength::Short, SummaryType::TLDR) => {
                "The summary must fit within one sentence."
            }
            (SummaryLength::Medium, SummaryType::TLDR) => {
                "The summary must fit within one short paragraph."
            }
            (SummaryLength::Long, SummaryType::TLDR) => {
                "The summary must fit within one paragraph."
            }
            (SummaryLength::Short, SummaryType::KeyPoints) => {
                "The summary must consist of no more than 3 bullet points."
            }
            (SummaryLength::Medium, SummaryType::KeyPoints) => {
                "The summary must consist of no more than 5 bullet points."
            }
            (SummaryLength::Long, SummaryType::KeyPoints) => {
                "The summary must consist of no more than 7 bullet points."
            }
            (SummaryLength::Short, SummaryType::Teaser) => {
                "The summary must fit within one sentence."
            }
            (SummaryLength::Medium, SummaryType::Teaser) => {
                "The summary must fit within one short paragraph."
            }
            (SummaryLength::Long, SummaryType::Teaser) => {
                "The summary must fit within one paragraph."
            }
            (SummaryLength::Short, SummaryType::Headline) => {
                "The headline must be concise, using a maximum of 12 words, and capture the essence of the text."
            }
            (SummaryLength::Medium, SummaryType::Headline) => {
                "The headline must be concise, using a maximum of 17 words, and capture the essence of the text."
            }
            (SummaryLength::Long, SummaryType::Headline) => {
                "The headline must be detailed, using a maximum of 22 words, and comprehensively capture the key themes of the text."
            }
        },
    );

    system_prompt.push_str("\n");

    system_prompt.push_str(match options.summary_format {
        SummaryFormat::Markdown => "The summary must be in valid Markdown syntax.",
        SummaryFormat::PlainText => {
            "The summary must not contain any formatting or markup language."
        }
    });

    system_prompt.push_str("TEXT:\n");
    system_prompt.push_str(input);
    system_prompt
}

#[cfg(test)]
mod tests {
    #[test]
    fn default_options() {
        let options = super::SummarizerOptions::default();
        assert_eq!(options.summary_type, super::SummaryType::TLDR);
        assert_eq!(options.summary_format, super::SummaryFormat::PlainText);
        assert_eq!(options.summary_length, super::SummaryLength::Medium);
        assert_eq!(options.shared_context, "")
    }
}
