mod egui;
mod excerpts;

#[cfg(not(target_arch = "wasm32"))]
use std::env;
#[cfg(not(target_arch = "wasm32"))]
use std::process::ExitCode;

#[cfg(not(target_arch = "wasm32"))]
fn usage(program: &str) -> String {
    format!("Usage: {program} sentence <id> | {program} display <id>")
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();

    let (Some(mode), Some(id_arg)) = (args.get(1), args.get(2)) else {
        eprintln!("{}", usage(&args[0]));
        return ExitCode::FAILURE;
    };

    let Ok(id) = id_arg.parse::<u32>() else {
        eprintln!("Invalid id: {id_arg}");
        return ExitCode::FAILURE;
    };

    match mode.as_str() {
        "sentence" => print_sentence(id),
        "display" => display_sentence(id),
        other => {
            eprintln!("Unknown mode: {other}");
            eprintln!("{}", usage(&args[0]));
            ExitCode::FAILURE
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn print_sentence(id: u32) -> ExitCode {
    match excerpts::sentences::run(id) {
        Some(sentence) => {
            let json =
                serde_json::to_string_pretty(&sentence).expect("failed to serialize sentence");
            println!("{json}");
            ExitCode::SUCCESS
        }
        None => {
            eprintln!("No sentence with id {id}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn display_sentence(id: u32) -> ExitCode {
    match excerpts::sentences::run(id) {
        Some(sentence) => match egui::select_word::run(sentence) {
            Ok(()) => ExitCode::SUCCESS,
            Err(err) => {
                eprintln!("Failed to display sentence: {err}");
                ExitCode::FAILURE
            }
        },
        None => {
            eprintln!("No sentence with id {id}");
            ExitCode::FAILURE
        }
    }
}

/// Used when the page URL has no (or an unparseable) `?id=`.
#[cfg(target_arch = "wasm32")]
const DEFAULT_SENTENCE_ID: u32 = 1;

/// Reads the sentence id from the page URL's `?id=` query parameter,
/// e.g. `?id=42`. Falls back to `DEFAULT_SENTENCE_ID` if it's missing
/// or not a valid id.
#[cfg(target_arch = "wasm32")]
fn sentence_id_from_url() -> u32 {
    let search = web_sys::window()
        .and_then(|window| window.location().search().ok())
        .unwrap_or_default();

    web_sys::UrlSearchParams::new_with_str(&search)
        .ok()
        .and_then(|params| params.get("id"))
        .and_then(|id| id.parse().ok())
        .unwrap_or(DEFAULT_SENTENCE_ID)
}

#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast;

    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();
    let id = sentence_id_from_url();
    let sentence =
        excerpts::sentences::run(id).unwrap_or_else(|| panic!("no sentence with id {id}"));

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("no window")
            .document()
            .expect("no document");
        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("failed to find #the_canvas_id")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("#the_canvas_id was not a canvas");

        document.set_title(&sentence.title());

        eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|_cc| Ok(Box::new(egui::select_word::SentenceApp::new(sentence)))),
            )
            .await
            .expect("failed to start eframe");
    });
}
