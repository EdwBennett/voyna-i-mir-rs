use eframe::egui;

const SENTENCE: &str = "The quick brown fox jumps over the lazy dog.";

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([520.0, 180.0])
            .with_resizable(false),
        ..Default::default()
    };

    eframe::run_native(
        "Clickable sentence",
        options,
        Box::new(|_cc| Ok(Box::new(SentenceApp::default()))),
    )
}

#[derive(Default)]
struct SentenceApp {
    last_clicked: Option<String>,
}

impl eframe::App for SentenceApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Click a word");

            ui.add_space(12.0);

            // Use a wrapping layout, so a longer sentence flows onto new lines.
            ui.horizontal_wrapped(|ui| {
                let mut character_index = 0;

                for word in SENTENCE.split_whitespace() {
                    // `character_index` is the index of this word's first
                    // Unicode character in SENTENCE.
                    let word_start = character_index;

                    let response = ui.add(
                        egui::Label::new(egui::RichText::new(word).size(20.0).underline())
                            .sense(egui::Sense::click()),
                    );

                    if response.clicked() {
                        self.word_clicked(word, word_start);
                    }

                    response.on_hover_text(format!("Character position: #{word_start}"));

                    // One space follows each word in this example.
                    character_index += word.chars().count() + 1;
                }
            });

            ui.add_space(20.0);

            if let Some(message) = &self.last_clicked {
                ui.label(message);
            }
        });
    }
}

impl SentenceApp {
    fn word_clicked(&mut self, word: &str, character_index: usize) {
        // Your routine receives the requested '#' character position.
        let position = format!("#{character_index}");

        self.last_clicked = Some(format!("Clicked “{word}” at character position {position}"));

        println!("word_clicked(word={word:?}, position={position})");
    }
}
