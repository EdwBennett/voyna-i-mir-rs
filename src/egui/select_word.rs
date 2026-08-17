use std::collections::HashSet;

use eframe::egui;

use crate::excerpts::sentences::{Sentence, WordToken};

/// Where a clicked word's gloss is shown.
#[allow(dead_code)]
enum GlossLayout {
    /// The most recently clicked word's gloss is shown below the sentence.
    Below,
    /// Each clicked word's gloss is shown inline, right after the word;
    /// clicking the word again hides it. Several can be open at once.
    Inline,
}

/// Change this to try a different layout.
const LAYOUT: GlossLayout = GlossLayout::Inline;

pub fn run(sentence: Sentence) -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([520.0, 220.0])
            .with_resizable(false),
        ..Default::default()
    };

    eframe::run_native(
        "Clickable sentence",
        options,
        Box::new(|_cc| Ok(Box::new(SentenceApp::new(sentence)))),
    )
}

struct SentenceApp {
    tokens: Vec<WordToken>,
    /// Indices into `tokens` whose gloss is currently shown.
    expanded: HashSet<usize>,
}

impl SentenceApp {
    fn new(sentence: Sentence) -> Self {
        Self {
            tokens: sentence.tokens(),
            expanded: HashSet::new(),
        }
    }

    /// Takes `expanded` rather than `&mut self` so callers can hold a
    /// borrow of `self.tokens` (from iterating it) at the same time.
    fn toggle_word(expanded: &mut HashSet<usize>, index: usize) {
        match LAYOUT {
            GlossLayout::Below => {
                expanded.clear();
                expanded.insert(index);
            }
            GlossLayout::Inline => {
                if !expanded.remove(&index) {
                    expanded.insert(index);
                }
            }
        }
    }
}

impl eframe::App for SentenceApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("Click a word");

            ui.add_space(12.0);

            // Use a wrapping layout, so a longer sentence flows onto new lines.
            ui.horizontal_wrapped(|ui| {
                // No automatic gap: punctuation should hug the word before it,
                // so spacing before words is added explicitly instead.
                ui.spacing_mut().item_spacing.x = 0.0;
                let mut first = true;

                for (index, token) in self.tokens.iter().enumerate() {
                    match token {
                        WordToken::Word { ru, ipa, en } => {
                            if !first {
                                ui.add_space(6.0);
                            }

                            let response = ui.add(
                                egui::Label::new(egui::RichText::new(ru).size(20.0).underline())
                                    .sense(egui::Sense::click()),
                            );

                            if response.clicked() {
                                Self::toggle_word(&mut self.expanded, index);
                            }

                            if matches!(LAYOUT, GlossLayout::Inline) && self.expanded.contains(&index) {
                                ui.add_space(6.0);
                                ui.label(egui::RichText::new(format!("({ipa}: {en})")).size(20.0).italics());
                            }
                        }
                        WordToken::Punct(text) => {
                            ui.label(egui::RichText::new(text).size(20.0));
                        }
                    }

                    first = false;
                }
            });

            if matches!(LAYOUT, GlossLayout::Below) {
                ui.add_space(20.0);

                if let Some((_, WordToken::Word { ru, ipa, en })) = self
                    .expanded
                    .iter()
                    .next()
                    .map(|&index| (index, &self.tokens[index]))
                {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 0.0;
                        ui.label(egui::RichText::new(ru).size(20.0));
                        ui.add_space(6.0);
                        ui.label(egui::RichText::new(ipa).size(20.0).italics());
                        ui.label(egui::RichText::new(format!(": {en}")).size(20.0));
                    });
                }
            }
        });
    }
}
