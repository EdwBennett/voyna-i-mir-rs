use serde::{Deserialize, Serialize};

const SENTENCES_YAML: &str = include_str!("sentences.yaml");

#[derive(Debug, Deserialize, Serialize)]
pub struct Sentence {
    pub id: u32,
    pub ru: String,
    pub ipa: String,
    pub en: String,
    pub words: String,
}

pub fn run(id: u32) -> Option<Sentence> {
    let sentences: Vec<Sentence> =
        serde_yaml_ng::from_str(SENTENCES_YAML).expect("failed to parse sentences.yaml");
    sentences.into_iter().find(|sentence| sentence.id == id)
}
