use eframe::egui;

/// Shown when the page URL has no `?page=`.
///
/// Deliberately says nothing about the other pages that exist, so that a
/// crawler or a curious visitor landing on the bare root URL has nothing to
/// follow.
pub struct HelloApp;

impl eframe::App for HelloApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.centered_and_justified(|ui| {
                ui.heading("Hello");
            });
        });
    }
}
