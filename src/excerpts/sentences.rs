use serde::{Deserialize, Serialize};

const SENTENCES_FILE: &str = "chapter_id_ru_en_ipa_words.yaml";
const SENTENCES_YAML: &str = include_str!("chapter_id_ru_en_ipa_words.yaml");
const VOLUME_PARTS_YAML: &str = include_str!("voyna-i-mir.yaml");

#[derive(Debug, Deserialize, Serialize)]
pub struct Sentence {
    pub chapter: String,
    pub id: u32,
    pub ru: String,
    pub ipa: String,
    pub ipa2: String,
    pub en: String,
    pub words: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct VolumePart {
    id: u32,
    vol_part: String,
    file: String,
}

pub fn run(id: u32) -> Option<Sentence> {
    let sentences: Vec<Sentence> = serde_yaml_ng::from_str(SENTENCES_YAML)
        .expect("failed to parse chapter_id_ru_en_ipa_words.yaml");
    sentences.into_iter().find(|sentence| sentence.id == id)
}

impl Sentence {
    /// Window title, e.g. "ТОМ ПЕРВЫЙ ЧАСТЬ ПЕРВАЯ IV".
    pub fn title(&self) -> String {
        let volume_parts: Vec<VolumePart> =
            serde_yaml_ng::from_str(VOLUME_PARTS_YAML).expect("failed to parse voyna-i-mir.yaml");
        let vol_part = volume_parts
            .into_iter()
            .find(|part| part.file == SENTENCES_FILE)
            .expect("voyna-i-mir.yaml should have an entry for chapter_id_ru_en_ipa_words.yaml")
            .vol_part;

        format!("{vol_part} {}", self.chapter)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WordToken {
    Word { ru: String, ipa: String, en: String },
    Punct(String),
}

impl Sentence {
    /// Parses `words` (e.g. "Так (thus / so) говорила (spoke / said) ...")
    /// into a sequence of glossed words and the punctuation between them, in
    /// the order they appear. Each word's IPA transcription is taken from
    /// `ipa`, whose words appear in the same order as `words`.
    pub fn tokens(&self) -> Vec<WordToken> {
        let mut ipa_words = Self::split_into_words(&self.ipa).into_iter();

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
                    ipa: ipa_words.next().unwrap_or_default(),
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

/// Punctuation marks that end a clause, for [`Sentence::clauses`].
const CLAUSE_BOUNDARIES: &[char] = &[',', ';', '-', '—', ':'];

/// A run of tokens between two clause boundaries (or the start/end of the
/// sentence).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Clause {
    pub tokens: Vec<WordToken>,
}

impl Clause {
    /// Renders the clause's tokens as a single line of Russian text,
    /// punctuation hugging the word before it rather than being
    /// space-separated.
    pub fn text(&self) -> String {
        let mut text = String::new();
        let mut first = true;

        for token in &self.tokens {
            match token {
                WordToken::Word { ru, .. } => {
                    if !first {
                        text.push(' ');
                    }
                    text.push_str(ru);
                }
                WordToken::Punct(punct) => text.push_str(punct),
            }
            first = false;
        }

        text
    }
}

impl Sentence {
    /// Splits [`Sentence::tokens`] into clauses, breaking after any
    /// [`WordToken::Punct`] token containing a character from
    /// [`CLAUSE_BOUNDARIES`]. The boundary punctuation stays attached to the
    /// clause it ends.
    pub fn clauses(&self) -> Vec<Clause> {
        let mut clauses = Vec::new();
        let mut current = Vec::new();

        for token in self.tokens() {
            let is_boundary =
                matches!(&token, WordToken::Punct(p) if p.contains(CLAUSE_BOUNDARIES));
            current.push(token);
            if is_boundary {
                clauses.push(Clause {
                    tokens: std::mem::take(&mut current),
                });
            }
        }

        if !current.is_empty() {
            clauses.push(Clause { tokens: current });
        }

        clauses
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_finds_sentence_by_id() {
        let sentence = run(1).expect("id 1 should exist in chapter_id_ru_en_ipa_words.yaml");
        assert_eq!(sentence.id, 1);
        assert!(sentence.ru.starts_with("Так говорила"));
    }

    #[test]
    fn title_combines_volume_part_and_chapter() {
        let sentence = run(4).expect("id 4 should exist in chapter_id_ru_en_ipa_words.yaml");
        assert_eq!(sentence.chapter, "IV");
        assert_eq!(sentence.title(), "ТОМ ПЕРВЫЙ ЧАСТЬ ПЕРВАЯ IV");
    }

    #[test]
    fn run_returns_none_for_unknown_id() {
        assert!(run(999_999).is_none());
    }

    #[test]
    fn tokens_parses_words_and_punctuation_in_order() {
        let sentence = run(1).expect("id 1 should exist in chapter_id_ru_en_ipa_words.yaml");
        let tokens = sentence.tokens();

        assert_eq!(
            tokens[0],
            WordToken::Word {
                ru: "Так".to_string(),
                ipa: "tɐk".to_string(),
                en: "thus / so".to_string()
            }
        );
        assert_eq!(
            tokens[1],
            WordToken::Word {
                ru: "говорила".to_string(),
                ipa: "ɡəvɐˈrʲilə".to_string(),
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
                ipa: "frʲɪˈlʲinə".to_string(),
                en: "maid of honor".to_string()
            }
        );
    }

    #[test]
    fn clauses_break_after_boundary_punctuation() {
        let sentence = run(1).expect("id 1 should exist in chapter_id_ru_en_ipa_words.yaml");
        let clauses = sentence.clauses();

        // "Шерер (Scherer)," ends the first clause: its last token is the
        // comma right after "Шерер".
        let first_clause_last = clauses[0]
            .tokens
            .last()
            .expect("clause should not be empty");
        assert_eq!(*first_clause_last, WordToken::Punct(",".to_string()));

        assert!(
            clauses.len() > 1,
            "sentence 1 should split into more than one clause"
        );

        // Every clause but a trailing one ending mid-sentence (no boundary
        // before it) should end on a boundary-containing Punct token.
        for clause in &clauses[..clauses.len() - 1] {
            let last = clause.tokens.last().expect("clause should not be empty");
            assert!(
                matches!(last, WordToken::Punct(p) if p.contains(CLAUSE_BOUNDARIES)),
                "clause should end on a boundary token, got {last:?}"
            );
        }
    }
}
