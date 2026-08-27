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

/// Voices a clause mp3 can be rendered in (see `src/mp3s.rs`), selected via
/// the `I`/`D` keys.
const IRINA: &str = "irina";
const DENIS: &str = "denis";

/// What a looping clip should do once its current playthrough ends.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Pending {
    /// Keep looping the same voice.
    KeepLooping,
    /// Go silent.
    Stop,
    /// Start looping the other voice, once.
    Switch(&'static str),
    /// Start looping the other voice, and keep alternating every loop after
    /// that (until something else - Space, `I`, `D` - overrides it).
    Alternate,
}

/// The other of [`IRINA`]/[`DENIS`].
fn other_voice(voice: &'static str) -> &'static str {
    if voice == IRINA { DENIS } else { IRINA }
}

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

/// Clause-level audio page: click a clause to select/highlight it and
/// immediately start looping the selected voice - clicking the already-
/// selected clause deselects it instead, stopping playback with no restart.
/// The left/right arrow keys (which wrap around the ends of the sentence)
/// only select/highlight, immediately stopping any playback in progress
/// without starting anything new - keyboard nav is unavailable on touch
/// devices anyway, so click alone (as used on GitHub Pages from iOS/
/// Android) needs to cover both selecting and starting playback. `I`/`D`
/// select the irina/denis voice and make sure it's looping the selected
/// clause (or the first clause, if none is selected) - letting whatever's
/// currently looping finish first if a different voice is playing. `A`
/// instead alternates the voice every loop (starting from the selected
/// voice, or continuing after the current playthrough finishes if
/// something's already playing) - it doesn't change the selected voice, so
/// alternation isn't remembered; it must be re-triggered with `A` each time.
/// Space toggles the selected voice's playback: starts it if idle, or lets
/// the current playthrough finish and then stops if playing - the selected
/// voice itself is remembered across stops and clause changes. A status
/// line below the clause text always shows what's selected/playing, for
/// orientation.
pub struct Page2App {
    clauses: Vec<Clause>,
    /// Index into `clauses` of the currently selected clause, if any.
    selected: Option<usize>,
    /// The voice `Space` plays/toggles, and that `I`/`D` last selected.
    /// Persists across stops and clause changes.
    selected_voice: &'static str,
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
    playing: Option<PlayingClip>,
}

#[cfg(not(target_arch = "wasm32"))]
struct PlayingClip {
    /// Dropping this stops its sound immediately.
    player: Player,
    sentence_id: u32,
    clause_num: usize,
    voice: &'static str,
    /// The currently-looping voice's decoded clip, kept around so each new
    /// loop can cheaply re-append it rather than re-decoding from disk.
    buffer: SamplesBuffer,
    pending: Pending,
}

#[cfg(not(target_arch = "wasm32"))]
impl NativeAudio {
    fn new() -> Self {
        match DeviceSinkBuilder::open_default_sink() {
            Ok(device) => Self {
                device: Some(device),
                playing: None,
            },
            Err(err) => {
                eprintln!("audio output unavailable: {err}");
                Self {
                    device: None,
                    playing: None,
                }
            }
        }
    }

    /// Reads and decodes `sentence_id`'s `clause_num`-th clause (1-based)
    /// in `voice`. `None` if that mp3 is missing or fails to decode.
    ///
    /// Decodes eagerly into a `SamplesBuffer` rather than keeping the
    /// `Decoder` around to loop lazily: rodio 0.22.2's mp3 decoder reports
    /// `current_span_len() == Some(0)` before its first sample is pulled,
    /// which `Source::buffered()` treats as "already exhausted" - producing
    /// permanent silence with no error if this source is ever looped via
    /// `repeat_infinite()` or similar.
    fn load_clip(sentence_id: u32, clause_num: usize, voice: &str) -> Option<SamplesBuffer> {
        let path = Path::new(clause_audio::RU_MP3S_DIR).join(
            clause_audio::clause_mp3_relative_path(sentence_id, clause_num, voice),
        );
        let bytes = std::fs::read(&path).ok()?;
        let decoder = Decoder::new(Cursor::new(bytes)).ok()?;
        let channels = decoder.channels();
        let sample_rate = decoder.sample_rate();
        Some(SamplesBuffer::new(
            channels,
            sample_rate,
            decoder.collect::<Vec<_>>(),
        ))
    }

    /// Hard-stops any playback immediately, with no grace period. Used when
    /// the selected clause changes, since the old clause's audio is no
    /// longer relevant.
    fn stop(&mut self) {
        self.playing = None;
    }

    /// Starts `voice` looping `sentence_id`'s `clause_num`-th clause
    /// (1-based) immediately. No-ops if that mp3 is missing or no audio
    /// output device is available.
    fn start(&mut self, sentence_id: u32, clause_num: usize, voice: &'static str) {
        let Some(device) = &self.device else {
            return;
        };
        let Some(buffer) = Self::load_clip(sentence_id, clause_num, voice) else {
            return;
        };

        let player = Player::connect_new(device.mixer());
        player.append(buffer.clone());
        self.playing = Some(PlayingClip {
            player,
            sentence_id,
            clause_num,
            voice,
            buffer,
            pending: Pending::KeepLooping,
        });
    }

    /// Handles an `I`/`D` press for `voice`: starts it looping immediately
    /// if nothing is playing; otherwise makes sure `voice` ends up looping
    /// once the current playthrough ends - immediately if it's already
    /// `voice` (cancelling any previously queued stop), or after a switch if
    /// a different voice is currently audible.
    fn select_voice(&mut self, sentence_id: u32, clause_num: usize, voice: &'static str) {
        match &mut self.playing {
            None => self.start(sentence_id, clause_num, voice),
            Some(playing) if playing.voice == voice => {
                playing.pending = Pending::KeepLooping;
            }
            Some(playing) => {
                playing.pending = Pending::Switch(voice);
            }
        }
    }

    /// Handles a Space press: starts `voice` looping immediately if nothing
    /// is playing, otherwise lets the current playthrough finish and then
    /// stops.
    fn toggle(&mut self, sentence_id: u32, clause_num: usize, voice: &'static str) {
        match &mut self.playing {
            None => self.start(sentence_id, clause_num, voice),
            Some(playing) => {
                playing.pending = Pending::Stop;
            }
        }
    }

    /// Handles an `A` press: starts `starting_voice` looping immediately if
    /// nothing is playing, then alternates voice every loop from then on
    /// (including if something else is already playing - the current
    /// playthrough finishes normally, then alternation begins).
    fn alternate(&mut self, sentence_id: u32, clause_num: usize, starting_voice: &'static str) {
        match &mut self.playing {
            None => {
                self.start(sentence_id, clause_num, starting_voice);
                if let Some(playing) = &mut self.playing {
                    playing.pending = Pending::Alternate;
                }
            }
            Some(playing) => {
                playing.pending = Pending::Alternate;
            }
        }
    }

    /// Advances looping playback: once the current playthrough ends,
    /// applies whatever was queued via [`Self::select_voice`], [`Self::toggle`],
    /// or [`Self::alternate`] - looping the same voice again, stopping,
    /// switching voice once, or alternating voice indefinitely. Call once
    /// per frame.
    fn update(&mut self) {
        let Some(playing) = self.playing.take() else {
            return;
        };
        if !playing.player.empty() {
            self.playing = Some(playing);
            return;
        }

        match playing.pending {
            Pending::KeepLooping => {
                playing.player.append(playing.buffer.clone());
                self.playing = Some(playing);
            }
            Pending::Stop => {}
            Pending::Switch(voice) => {
                self.start(playing.sentence_id, playing.clause_num, voice);
            }
            Pending::Alternate => {
                self.start(
                    playing.sentence_id,
                    playing.clause_num,
                    other_voice(playing.voice),
                );
                if let Some(now_playing) = &mut self.playing {
                    now_playing.pending = Pending::Alternate;
                }
            }
        }
    }

    /// The voice, clause number (1-based), and pending action of whatever's
    /// currently looping, if anything - for the status line.
    fn status(&self) -> Option<(&'static str, usize, Pending)> {
        self.playing
            .as_ref()
            .map(|playing| (playing.voice, playing.clause_num, playing.pending))
    }
}

/// Web (wasm) clause-audio playback, backed by an `HTMLAudioElement`.
/// `egui`/`eframe` only draw to the canvas, so playback goes through
/// `web_sys` directly rather than any egui widget.
#[cfg(target_arch = "wasm32")]
#[derive(Default)]
struct WebAudio {
    /// Present while a clause mp3 is looping.
    playing: Option<PlayingClip>,
}

#[cfg(target_arch = "wasm32")]
struct PlayingClip {
    element: HtmlAudioElement,
    sentence_id: u32,
    clause_num: usize,
    voice: &'static str,
    pending: Pending,
}

#[cfg(target_arch = "wasm32")]
impl WebAudio {
    /// Hard-stops any playback immediately, with no grace period. Used when
    /// the selected clause changes, since the old clause's audio is no
    /// longer relevant.
    fn stop(&mut self) {
        if let Some(playing) = self.playing.take() {
            let _ = playing.element.pause();
        }
    }

    /// Starts `voice` looping `sentence_id`'s `clause_num`-th clause
    /// (1-based) immediately. No-ops if the element or playback can't be
    /// created (e.g. the mp3 is missing - trunk's `copy-dir` directive in
    /// index.html puts `src/ru-mp3s/` at `ru-mp3s/` relative to the page,
    /// which resolves against the `<base data-trunk-public-url>` tag
    /// regardless of deploy path).
    fn start(&mut self, sentence_id: u32, clause_num: usize, voice: &'static str) {
        let src = format!(
            "ru-mp3s/{}",
            clause_audio::clause_mp3_relative_path(sentence_id, clause_num, voice)
        );
        let Ok(element) = HtmlAudioElement::new_with_src(&src) else {
            return;
        };
        let _ = element.play();
        self.playing = Some(PlayingClip {
            element,
            sentence_id,
            clause_num,
            voice,
            pending: Pending::KeepLooping,
        });
    }

    /// Handles an `I`/`D` press for `voice`: starts it looping immediately
    /// if nothing is playing; otherwise makes sure `voice` ends up looping
    /// once the current playthrough ends - immediately if it's already
    /// `voice` (cancelling any previously queued stop), or after a switch if
    /// a different voice is currently audible.
    fn select_voice(&mut self, sentence_id: u32, clause_num: usize, voice: &'static str) {
        match &mut self.playing {
            None => self.start(sentence_id, clause_num, voice),
            Some(playing) if playing.voice == voice => {
                playing.pending = Pending::KeepLooping;
            }
            Some(playing) => {
                playing.pending = Pending::Switch(voice);
            }
        }
    }

    /// Handles a Space press: starts `voice` looping immediately if nothing
    /// is playing, otherwise lets the current playthrough finish and then
    /// stops.
    fn toggle(&mut self, sentence_id: u32, clause_num: usize, voice: &'static str) {
        match &mut self.playing {
            None => self.start(sentence_id, clause_num, voice),
            Some(playing) => {
                playing.pending = Pending::Stop;
            }
        }
    }

    /// Handles an `A` press: starts `starting_voice` looping immediately if
    /// nothing is playing, then alternates voice every loop from then on
    /// (including if something else is already playing - the current
    /// playthrough finishes normally, then alternation begins).
    fn alternate(&mut self, sentence_id: u32, clause_num: usize, starting_voice: &'static str) {
        match &mut self.playing {
            None => {
                self.start(sentence_id, clause_num, starting_voice);
                if let Some(playing) = &mut self.playing {
                    playing.pending = Pending::Alternate;
                }
            }
            Some(playing) => {
                playing.pending = Pending::Alternate;
            }
        }
    }

    /// Advances looping playback: once the current playthrough ends,
    /// applies whatever was queued via [`Self::select_voice`], [`Self::toggle`],
    /// or [`Self::alternate`] - looping the same voice again, stopping,
    /// switching voice once, or alternating voice indefinitely. Call once
    /// per frame.
    fn update(&mut self) {
        let Some(playing) = self.playing.take() else {
            return;
        };
        if !playing.element.ended() {
            self.playing = Some(playing);
            return;
        }

        match playing.pending {
            Pending::KeepLooping => {
                playing.element.set_current_time(0.0);
                let _ = playing.element.play();
                self.playing = Some(playing);
            }
            Pending::Stop => {}
            Pending::Switch(voice) => {
                self.start(playing.sentence_id, playing.clause_num, voice);
            }
            Pending::Alternate => {
                self.start(
                    playing.sentence_id,
                    playing.clause_num,
                    other_voice(playing.voice),
                );
                if let Some(now_playing) = &mut self.playing {
                    now_playing.pending = Pending::Alternate;
                }
            }
        }
    }

    /// The voice, clause number (1-based), and pending action of whatever's
    /// currently looping, if anything - for the status line.
    fn status(&self) -> Option<(&'static str, usize, Pending)> {
        self.playing
            .as_ref()
            .map(|playing| (playing.voice, playing.clause_num, playing.pending))
    }
}

impl Page2App {
    pub fn new(sentence: Sentence) -> Self {
        Self {
            sentence_id: sentence.id,
            clauses: sentence.clauses(),
            selected: None,
            selected_voice: IRINA,
            #[cfg(not(target_arch = "wasm32"))]
            audio: NativeAudio::new(),
            #[cfg(target_arch = "wasm32")]
            audio: WebAudio::default(),
        }
    }

    /// The 1-based clause number Space/`I`/`D` playback targets: the
    /// selected clause, or the first clause if none is selected yet.
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
        self.audio.update();

        let (arrow_right, arrow_left, space, key_i, key_d, key_a, ctrl_w) = ui.input(|i| {
            (
                i.key_pressed(egui::Key::ArrowRight),
                i.key_pressed(egui::Key::ArrowLeft),
                i.key_pressed(egui::Key::Space),
                i.key_pressed(egui::Key::I),
                i.key_pressed(egui::Key::D),
                i.key_pressed(egui::Key::A),
                i.modifiers.ctrl && i.key_pressed(egui::Key::W),
            )
        });

        if ctrl_w {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
        }

        if arrow_right {
            self.audio.stop();
            self.selected = Self::next_clause_index(self.clauses.len(), self.selected);
        }

        if arrow_left {
            self.audio.stop();
            self.selected = Self::prev_clause_index(self.clauses.len(), self.selected);
        }

        if let Some(clause_num) = self.current_clause_num() {
            if key_i {
                self.selected_voice = IRINA;
                self.audio.select_voice(self.sentence_id, clause_num, IRINA);
            }
            if key_d {
                self.selected_voice = DENIS;
                self.audio.select_voice(self.sentence_id, clause_num, DENIS);
            }
            if space {
                self.audio
                    .toggle(self.sentence_id, clause_num, self.selected_voice);
            }
            if key_a {
                self.audio
                    .alternate(self.sentence_id, clause_num, self.selected_voice);
            }
        }

        let status = self.audio.status();
        if status.is_some() {
            // egui/eframe only repaints in response to input by default, but
            // looping playback needs `update()` to keep running each frame
            // (to notice a clip ending and re-append/switch/stop) even with
            // no mouse or keyboard activity.
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_millis(50));
        }

        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading(
                "Click a clause to play it; space toggles, i/d selects voice, a alternates",
            );

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
                            if is_selected {
                                self.selected = None;
                            } else {
                                self.selected = Some(index);
                                self.audio
                                    .start(self.sentence_id, index + 1, self.selected_voice);
                            }
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

            ui.add_space(12.0);

            let status_text = match status {
                Some((voice, clause_num, Pending::KeepLooping)) => {
                    format!("Playing {voice}, clause {clause_num}")
                }
                Some((voice, clause_num, Pending::Stop)) => {
                    format!("Playing {voice}, clause {clause_num} — stopping after this loop")
                }
                Some((voice, clause_num, Pending::Switch(next_voice))) => {
                    format!(
                        "Playing {voice}, clause {clause_num} — switching to {next_voice} after this loop"
                    )
                }
                Some((voice, clause_num, Pending::Alternate)) => {
                    format!(
                        "Playing {voice}, clause {clause_num} — alternating with {}",
                        other_voice(voice)
                    )
                }
                None => match self.selected {
                    Some(index) => format!(
                        "Clause {} selected — space plays {}",
                        index + 1,
                        self.selected_voice
                    ),
                    None => "No clause selected".to_string(),
                },
            };
            ui.weak(status_text);
        });
    }
}
