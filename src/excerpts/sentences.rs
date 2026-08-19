use serde::{Deserialize, Serialize};

const SENTENCES_YAML: &str = include_str!("Volume-1-Part-1.yaml");

#[derive(Debug, Deserialize, Serialize)]
pub struct Sentence {
    pub chapter: String,
    pub id: u32,
    pub ru: String,
    pub roman: String,
    pub en: String,
    pub words: String,
}

pub fn run(id: u32) -> Option<Sentence> {
    let sentences: Vec<Sentence> =
        serde_yaml_ng::from_str(SENTENCES_YAML).expect("failed to parse Volume-1-Part-1.yaml");
    sentences.into_iter().find(|sentence| sentence.id == id)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WordToken {
    Word { ru: String, roman: String, en: String },
    Punct(String),
}

impl Sentence {
    /// Parses `words` (e.g. "Так (thus / so) говорила (spoke / said) ...")
    /// into a sequence of glossed words and the punctuation between them, in
    /// the order they appear. Each word's romanization is taken from
    /// `roman`, whose words appear in the same order as `words`.
    pub fn tokens(&self) -> Vec<WordToken> {
        let mut romanized_words = Self::split_into_words(&self.roman).into_iter();

        let mut tokens = Vec::new();
        let mut rest = self.words.as_str();

        while let Some(open) = rest.find('(') {
            let chunk = rest[..open].trim();
            let word_start = chunk
                .find(|c: char| c.is_alphanumeric())
                .unwrap_or(chunk.len());
            let punct = chunk[..word_start].trim();
            let word = &chunk[word_start..];

            if !punct.is_empty() {
                tokens.push(WordToken::Punct(punct.to_string()));
            }

            let after_open = &rest[open + 1..];
            let Some(close) = after_open.find(')') else {
                break;
            };
            let gloss = after_open[..close].trim();

            if !word.is_empty() {
                tokens.push(WordToken::Word {
                    ru: word.to_string(),
                    roman: romanized_words.next().unwrap_or_default(),
                    en: gloss.to_string(),
                });
            }

            rest = &after_open[close + 1..];
        }

        let trailing = rest.trim();
        if !trailing.is_empty() {
            tokens.push(WordToken::Punct(trailing.to_string()));
        }

        tokens
    }

    fn split_into_words(text: &str) -> Vec<String> {
        text.split_whitespace()
            .map(|word| {
                word.trim_matches(|c: char| !c.is_alphanumeric())
                    .to_string()
            })
            .filter(|word| !word.is_empty())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_finds_sentence_by_id() {
        let sentence = run(1).expect("id 1 should exist in Volume-1-Part-1.yaml");
        assert_eq!(sentence.id, 1);
        assert!(sentence.ru.starts_with("Так говорила"));
    }

    #[test]
    fn run_returns_none_for_unknown_id() {
        assert!(run(999_999).is_none());
    }

    #[test]
    fn tokens_parses_words_and_punctuation_in_order() {
        let sentence = run(1).expect("id 1 should exist in Volume-1-Part-1.yaml");
        let tokens = sentence.tokens();

        assert_eq!(
            tokens[0],
            WordToken::Word {
                ru: "Так".to_string(),
                roman: "Tak".to_string(),
                en: "thus / so".to_string()
            }
        );
        assert_eq!(
            tokens[1],
            WordToken::Word {
                ru: "говорила".to_string(),
                roman: "govorila".to_string(),
                en: "spoke / said".to_string()
            }
        );

        // The comma after "Шерер (Scherer)," is its own token between the two words.
        let sherer = tokens
            .iter()
            .position(|t| matches!(t, WordToken::Word { ru, .. } if ru == "Шерер"))
            .expect("Шерер should be present");
        assert_eq!(tokens[sherer + 1], WordToken::Punct(",".to_string()));
        assert_eq!(
            tokens[sherer + 2],
            WordToken::Word {
                ru: "фрейлина".to_string(),
                roman: "freylina".to_string(),
                en: "maid of honor".to_string()
            }
        );
    }
}
