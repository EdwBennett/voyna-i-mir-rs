use eframe::egui;

use crate::clause_audio;
use crate::excerpts::sentences::{Clause, Sentence};

#[cfg(not(target_arch = "wasm32"))]
use std::io::Cursor;
#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;

#[cfg(not(target_arch = "wasm32"))]
use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player, Source, buffer::SamplesBuffer};

#[cfg(target_arch = "wasm32")]
use web_sys::HtmlAudioElement;

const TEXT_SIZE: f32 = 24.0;

/// Voice clause playback uses. Both `denis` and `irina` mp3s exist per
/// clause (see `src/mp3s.rs`); irina is the one wired up to the UI.
const VOICE: &str = "irina";

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

/// Clause-level audio page: click a clause (or use the left/right arrow
/// keys, which wrap around the ends of the sentence) to select/highlight
/// it, which also stops any playback in progress. Space starts looping the
/// selected clause's mp3 - or the first clause's, if none is selected -
/// until space is pressed again.
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

/// Native (non-wasm) clause-audio playback, backed by `rodio`.
#[cfg(not(target_arch = "wasm32"))]
struct NativeAudio {
    /// Kept alive for as long as playback should be possible - dropping it
    /// silences any player connected to its mixer.
    device: Option<MixerDeviceSink>,
    /// Present while a clause mp3 is looping.
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

    /// Stops playback if a clause mp3 is currently looping.
    fn stop(&mut self) {
        if let Some(player) = self.player.take() {
            player.stop();
        }
    }

    /// Stops playback if a clause mp3 is currently looping, otherwise
    /// starts looping `sentence_id`'s `clause_num`-th clause (1-based) in
    /// `voice` until stopped. No-ops if that mp3 is missing or no audio
    /// output device is available.
    fn toggle(&mut self, sentence_id: u32, clause_num: usize, voice: &str) {
        if self.player.is_some() {
            self.stop();
            return;
        }

        let Some(device) = &self.device else {
            return;
        };

        let path = Path::new(clause_audio::RU_MP3S_DIR).join(
            clause_audio::clause_mp3_relative_path(sentence_id, clause_num, voice),
        );
        let Ok(bytes) = std::fs::read(&path) else {
            return;
        };
        let Ok(decoder) = Decoder::new(Cursor::new(bytes)) else {
            return;
        };
        // Decode eagerly into a SamplesBuffer rather than calling
        // `repeat_infinite()` on the Decoder directly: rodio 0.22.2's mp3
        // decoder reports `current_span_len() == Some(0)` before its first
        // sample is pulled, which `Source::buffered()` (which
        // `repeat_infinite()` relies on internally) treats as "already
        // exhausted" - producing permanent silence with no error.
        let channels = decoder.channels();
        let sample_rate = decoder.sample_rate();
        let source = SamplesBuffer::new(channels, sample_rate, decoder.collect::<Vec<_>>());

        let player = Player::connect_new(device.mixer());
        player.append(source.repeat_infinite());
        self.player = Some(player);
    }
}

/// Web (wasm) clause-audio playback, backed by an `HTMLAudioElement`.
/// `egui`/`eframe` only draw to the canvas, so playback goes through
/// `web_sys` directly rather than any egui widget.
#[cfg(target_arch = "wasm32")]
#[derive(Default)]
struct WebAudio {
    /// Present while a clause mp3 is looping.
    element: Option<HtmlAudioElement>,
}

#[cfg(target_arch = "wasm32")]
impl WebAudio {
    /// Stops playback if a clause mp3 is currently looping.
    fn stop(&mut self) {
        if let Some(element) = self.element.take() {
            let _ = element.pause();
            element.set_current_time(0.0);
        }
    }

    /// Stops playback if a clause mp3 is currently looping, otherwise
    /// starts looping `sentence_id`'s `clause_num`-th clause (1-based) in
    /// `voice` until stopped. No-ops if the element or playback can't be
    /// created (e.g. the mp3 is missing - trunk's `copy-dir` directive in
    /// index.html puts `src/ru-mp3s/` at `ru-mp3s/` relative to the page,
    /// which resolves against the `<base data-trunk-public-url>` tag
    /// regardless of deploy path).
    fn toggle(&mut self, sentence_id: u32, clause_num: usize, voice: &str) {
        if self.element.is_some() {
            self.stop();
            return;
        }

        let src = format!(
            "ru-mp3s/{}",
            clause_audio::clause_mp3_relative_path(sentence_id, clause_num, voice)
        );
        let Ok(element) = HtmlAudioElement::new_with_src(&src) else {
            return;
        };
        element.set_loop(true);
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

    /// The 1-based clause number space-bar playback targets: the selected
    /// clause, or the first clause if none is selected yet.
    fn current_clause_num(&self) -> Option<usize> {
        if self.clauses.is_empty() {
            return None;
        }
        Some(self.selected.unwrap_or(0) + 1)
    }

    /// Index of the clause after `after` (or the first clause if `after` is
    /// `None`), wrapping around the end of the sentence.
    fn next_clause_index(len: usize, after: Option<usize>) -> Option<usize> {
        if len == 0 {
            return None;
        }
        Some(after.map_or(0, |i| (i + 1) % len))
    }

    /// Index of the clause before `before` (or the last clause if `before`
    /// is `None`), wrapping around the start of the sentence.
    fn prev_clause_index(len: usize, before: Option<usize>) -> Option<usize> {
        if len == 0 {
            return None;
        }
        Some(before.map_or(len - 1, |i| (i + len - 1) % len))
    }
}

impl eframe::App for Page2App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let (arrow_right, arrow_left, space) = ui.input(|i| {
            (
                i.key_pressed(egui::Key::ArrowRight),
                i.key_pressed(egui::Key::ArrowLeft),
                i.key_pressed(egui::Key::Space),
            )
        });

        if arrow_right {
            self.audio.stop();
            self.selected = Self::next_clause_index(self.clauses.len(), self.selected);
        }

        if arrow_left {
            self.audio.stop();
            self.selected = Self::prev_clause_index(self.clauses.len(), self.selected);
        }

        if space && let Some(clause_num) = self.current_clause_num() {
            self.audio.toggle(self.sentence_id, clause_num, VOICE);
        }

        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("Click a clause (or left/right-arrow); space plays it on repeat");

            ui.add_space(12.0);

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;

                    for (index, clause) in self.clauses.iter().enumerate() {
                        let is_selected = self.selected == Some(index);

                        let mut text = egui::RichText::new(clause.text()).size(TEXT_SIZE);
                        if is_selected {
                            text = text.background_color(ui.visuals().selection.bg_fill);
                        }

                        let response = ui.add(egui::Label::new(text).sense(egui::Sense::click()));

                        if response.clicked() {
                            self.audio.stop();
                            self.selected = if is_selected { None } else { Some(index) };
                        }

                        // Keep the newly-selected clause visible when it was
                        // reached via keyboard nav rather than a click (which
                        // is already visible, having just been clicked).
                        if is_selected && (arrow_right || arrow_left) {
                            response.scroll_to_me(Some(egui::Align::Center));
                        }

                        ui.add_space(6.0);
                    }
                });
            });
        });
    }
}
