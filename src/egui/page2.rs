use eframe::egui;

use crate::excerpts::sentences::{Clause, Sentence, WordToken};

#[cfg(not(target_arch = "wasm32"))]
use std::io::Cursor;

#[cfg(not(target_arch = "wasm32"))]
use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player};

#[cfg(target_arch = "wasm32")]
use web_sys::HtmlAudioElement;

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

/// Clause-level clause-audio page: click a clause to select it. Per-clause
/// playback (looping each clause's mp3 until clicked again) is not
/// implemented yet, since those mp3s don't exist - they'll be generated
/// later via piper-voices. For now, space plays the whole sentence's mp3
/// once, as a first test of the audio playback infrastructure.
pub struct Page2App {
    clauses: Vec<Clause>,
    /// Index into `clauses` of the currently selected clause, if any.
    selected: Option<usize>,
    sentence_id: u32,
    #[cfg(not(target_arch = "wasm32"))]
    audio: NativeAudio,
    #[cfg(target_arch = "wasm32")]
    audio: WebAudio,
}

/// Native (non-wasm) sentence-audio playback, backed by `rodio`.
#[cfg(not(target_arch = "wasm32"))]
struct NativeAudio {
    /// Kept alive for as long as playback should be possible - dropping it
    /// silences any player connected to its mixer.
    device: Option<MixerDeviceSink>,
    /// Present while a sentence mp3 is playing.
    player: Option<Player>,
}

#[cfg(not(target_arch = "wasm32"))]
impl NativeAudio {
    fn new() -> Self {
        match DeviceSinkBuilder::open_default_sink() {
            Ok(device) => Self {
                device: Some(device),
                player: None,
            },
            Err(err) => {
                eprintln!("audio output unavailable: {err}");
                Self {
                    device: None,
                    player: None,
                }
            }
        }
    }

    /// Clears `player` once its one-shot playback has finished on its own.
    fn forget_finished(&mut self) {
        if self.player.as_ref().is_some_and(Player::empty) {
            self.player = None;
        }
    }

    fn is_active(&self) -> bool {
        self.player.is_some()
    }

    /// Stops playback if a sentence mp3 is currently playing, otherwise
    /// starts playing `sentence_id`'s mp3 once. No-ops if that mp3 is
    /// missing or no audio output device is available.
    fn toggle(&mut self, sentence_id: u32) {
        if let Some(player) = self.player.take() {
            player.stop();
            return;
        }

        let Some(device) = &self.device else {
            return;
        };

        let path = format!(
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/ru-mp3s/voynaimir_{:03}.mp3"
            ),
            sentence_id
        );
        let Ok(bytes) = std::fs::read(&path) else {
            return;
        };
        let Ok(source) = Decoder::new(Cursor::new(bytes)) else {
            return;
        };

        let player = Player::connect_new(device.mixer());
        player.append(source);
        self.player = Some(player);
    }
}

/// Web (wasm) sentence-audio playback, backed by an `HTMLAudioElement`.
/// `egui`/`eframe` only draw to the canvas, so playback goes through
/// `web_sys` directly rather than any egui widget.
#[cfg(target_arch = "wasm32")]
#[derive(Default)]
struct WebAudio {
    /// Present while a sentence mp3 is playing.
    element: Option<HtmlAudioElement>,
}

#[cfg(target_arch = "wasm32")]
impl WebAudio {
    /// Clears `element` once its one-shot playback has finished on its own.
    fn forget_finished(&mut self) {
        if self.element.as_ref().is_some_and(|element| element.ended()) {
            self.element = None;
        }
    }

    fn is_active(&self) -> bool {
        self.element.is_some()
    }

    /// Stops playback if a sentence mp3 is currently playing, otherwise
    /// starts playing `sentence_id`'s mp3 once. No-ops if the element or
    /// playback can't be created (e.g. the mp3 is missing - trunk's
    /// `copy-dir` directive in index.html puts `src/ru-mp3s/` at
    /// `ru-mp3s/` relative to the page, which resolves against the
    /// `<base data-trunk-public-url>` tag regardless of deploy path).
    fn toggle(&mut self, sentence_id: u32) {
        if let Some(element) = self.element.take() {
            let _ = element.pause();
            element.set_current_time(0.0);
            return;
        }

        let src = format!("ru-mp3s/voynaimir_{sentence_id:03}.mp3");
        let Ok(element) = HtmlAudioElement::new_with_src(&src) else {
            return;
        };
        let _ = element.play();
        self.element = Some(element);
    }
}

impl Page2App {
    pub fn new(sentence: Sentence) -> Self {
        Self {
            sentence_id: sentence.id,
            clauses: sentence.clauses(),
            selected: None,
            #[cfg(not(target_arch = "wasm32"))]
            audio: NativeAudio::new(),
            #[cfg(target_arch = "wasm32")]
            audio: WebAudio::default(),
        }
    }
}

impl eframe::App for Page2App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.audio.forget_finished();

        if ui.input(|i| i.key_pressed(egui::Key::Space)) {
            self.audio.toggle(self.sentence_id);
        }

        // Keep repainting while a sound is playing so `forget_finished`
        // notices promptly once it ends, without needing more input.
        if self.audio.is_active() {
            ui.ctx().request_repaint();
        }

        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("Click a clause (space plays the sentence audio)");

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
