use eframe::egui;

use crate::excerpts::sentences::{Clause, Sentence, WordToken};

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
        Box::new(|_cc| Ok(Box::new(Page2App::new(sentence)))),
    )
}

/// Renders a clause's tokens as a single line of text, punctuation hugging
/// the word before it (matching how `select_word` renders tokens).
fn clause_text(clause: &Clause) -> String {
    let mut text = String::new();
    let mut first = true;

    for token in &clause.tokens {
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

/// Clause-level clause-audio page: click a clause to select it. Playback
/// (loop the clause's mp3 until clicked again) is not implemented yet -
/// this only tracks which clause is selected. Arrow-key/space navigation is
/// still undecided and intentionally left unhandled.
pub struct Page2App {
    clauses: Vec<Clause>,
    /// Index into `clauses` of the currently selected clause, if any.
    selected: Option<usize>,
}

impl Page2App {
    pub fn new(sentence: Sentence) -> Self {
        Self {
            clauses: sentence.clauses(),
            selected: None,
        }
    }
}

impl eframe::App for Page2App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("Click a clause");

            ui.add_space(12.0);

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;

                    for (index, clause) in self.clauses.iter().enumerate() {
                        let is_selected = self.selected == Some(index);

                        let mut text = egui::RichText::new(clause_text(clause)).size(TEXT_SIZE);
                        if is_selected {
                            text = text.background_color(ui.visuals().selection.bg_fill);
                        }

                        let response = ui.add(egui::Label::new(text).sense(egui::Sense::click()));

                        if response.clicked() {
                            self.selected = if is_selected { None } else { Some(index) };
                        }

                        ui.add_space(6.0);
                    }
                });
            });
        });
    }
}
