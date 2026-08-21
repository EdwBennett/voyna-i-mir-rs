use std::collections::HashSet;

use eframe::egui;

use crate::excerpts::sentences::{Sentence, WordToken};

const TEXT_SIZE: f32 = 24.0;

#[cfg(not(target_arch = "wasm32"))]
pub fn run(sentence: Sentence) -> eframe::Result<()> {
    let title = sentence.title();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([624.0, 264.0])
            .with_resizable(false),
        ..Default::default()
    };

    eframe::run_native(
        &title,
        options,
        Box::new(|_cc| Ok(Box::new(SentenceApp::new(sentence)))),
    )
}

pub struct SentenceApp {
    tokens: Vec<WordToken>,
    /// Indices into `tokens` whose gloss is currently shown.
    expanded: HashSet<usize>,
    /// Index into `tokens` of the word currently selected via keyboard
    /// (or last clicked).
    selected: Option<usize>,
}

impl SentenceApp {
    pub fn new(sentence: Sentence) -> Self {
        Self {
            tokens: sentence.tokens(),
            expanded: HashSet::new(),
            selected: None,
        }
    }

    /// Takes `expanded` rather than `&mut self` so callers can hold a
    /// borrow of `self.tokens` (from iterating it) at the same time.
    fn toggle_word(expanded: &mut HashSet<usize>, index: usize) {
        if !expanded.remove(&index) {
            expanded.insert(index);
        }
    }

    /// Index of the next `Word` token after `after` (or the first `Word`
    /// token if `after` is `None`), wrapping around the end of the sentence.
    fn next_word_index(tokens: &[WordToken], after: Option<usize>) -> Option<usize> {
        let len = tokens.len();
        if len == 0 {
            return None;
        }

        let start = after.map_or(0, |i| i + 1);
        (0..len)
            .map(|offset| (start + offset) % len)
            .find(|&i| matches!(tokens[i], WordToken::Word { .. }))
    }

    /// Index of the previous `Word` token before `before` (or the last `Word`
    /// token if `before` is `None`), wrapping around the start of the sentence.
    fn prev_word_index(tokens: &[WordToken], before: Option<usize>) -> Option<usize> {
        let len = tokens.len();
        if len == 0 {
            return None;
        }

        let start = before.map_or(len - 1, |i| (i + len - 1) % len);
        (0..len)
            .map(|offset| (start + len - offset) % len)
            .find(|&i| matches!(tokens[i], WordToken::Word { .. }))
    }
}

impl eframe::App for SentenceApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let (arrow_right, arrow_left, space, ctrl_w) = ui.input(|i| {
            (
                i.key_pressed(egui::Key::ArrowRight),
                i.key_pressed(egui::Key::ArrowLeft),
                i.key_pressed(egui::Key::Space),
                i.modifiers.ctrl && i.key_pressed(egui::Key::W),
            )
        });

        if arrow_right {
            self.selected = Self::next_word_index(&self.tokens, self.selected);
        }

        if arrow_left {
            self.selected = Self::prev_word_index(&self.tokens, self.selected);
        }

        if ctrl_w {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
        }

        if space && let Some(index) = self.selected {
            Self::toggle_word(&mut self.expanded, index);
        }

        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("Click a word (or left/right-arrow, space)");

            ui.add_space(12.0);

            egui::ScrollArea::vertical().show(ui, |ui| {
                // Use a wrapping layout, so a longer sentence flows onto new lines.
                ui.horizontal_wrapped(|ui| {
                    // No automatic gap: punctuation should hug the word before it,
                    // so spacing before words is added explicitly instead.
                    ui.spacing_mut().item_spacing.x = 0.0;
                    let mut first = true;

                    for (index, token) in self.tokens.iter().enumerate() {
                        match token {
                            WordToken::Word { ru, roman, en } => {
                                if !first {
                                    ui.add_space(6.0);
                                }

                                let is_selected = self.selected == Some(index);

                                let mut text = egui::RichText::new(ru).size(TEXT_SIZE).underline();
                                if is_selected {
                                    text = text.background_color(ui.visuals().selection.bg_fill);
                                }

                                let response =
                                    ui.add(egui::Label::new(text).sense(egui::Sense::click()));

                                if response.clicked() {
                                    self.selected = Some(index);
                                    Self::toggle_word(&mut self.expanded, index);
                                }

                                if self.expanded.contains(&index) {
                                    ui.add_space(6.0);
                                    ui.label(
                                        egui::RichText::new(format!("({roman}: {en})"))
                                            .size(TEXT_SIZE)
                                            .italics(),
                                    );
                                }
                            }
                            WordToken::Punct(text) => {
                                ui.label(egui::RichText::new(text).size(TEXT_SIZE));
                            }
                        }

                        first = false;
                    }
                });
            });
        });
    }
}
