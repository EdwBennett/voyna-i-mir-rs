mod excerpts;

use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();

    let Some(id_arg) = args.get(1) else {
        eprintln!("Usage: {} <id>", args[0]);
        return ExitCode::FAILURE;
    };

    let Ok(id) = id_arg.parse::<u32>() else {
        eprintln!("Invalid id: {id_arg}");
        return ExitCode::FAILURE;
    };

    match excerpts::sentences::run(id) {
        Some(sentence) => {
            let json = serde_json::to_string_pretty(&sentence).expect("failed to serialize sentence");
            println!("{json}");
            ExitCode::SUCCESS
        }
        None => {
            eprintln!("No sentence with id {id}");
            ExitCode::FAILURE
        }
    }
}
