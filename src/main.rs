// cargo run -- sentence 1   -> prints the sentence with id 1 as JSON
// cargo run -- display 1    -> opens the egui window for the sentence with id 1

mod egui;
mod excerpts;

use std::env;
use std::process::ExitCode;

fn usage(program: &str) -> String {
    format!("Usage: {program} sentence <id> | {program} display <id>")
}

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
