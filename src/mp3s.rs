//! `cargo run -- mp3s <id>`: renders one mp3 per clause per Russian `piper`
//! voice (denis, irina) for a sentence, via `piper` (text -> raw PCM) piped
//! into `ffmpeg` (PCM -> mp3). Mirrors the pipeline already proven out in
//! `make_ru_mp3.rs` from a sibling project.
//!
//! Files land under `src/ru-mp3s/vol-1-part-1/{sentence_id:03}/`, named
//! `{clause:02}_{voice}.mp3` (1-based clause numbering), and are always
//! overwritten.
//!
//! Setup: `piper` and `ffmpeg` must be on `$PATH`, and each voice's .onnx +
//! .onnx.json pair must exist under
//! ~/.local/share/piper-voices/ru/ru_RU/<voice>/medium/.

use std::env;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode, Output, Stdio};

use crate::clause_audio;
use crate::excerpts::sentences;

/// Audio sample rate (Hz) produced by the voice models and fed to `ffmpeg`.
const SAMPLE_RATE: u32 = 22_050;

/// Duration of silence rendered before each clause's speech, so playback
/// devices that take a moment to wake up don't clip the start of the audio.
const LEAD_IN_SECONDS: f64 = 0.5;

const VOICES: [&str; 2] = ["denis", "irina"];

fn voice_model_paths(home: &Path, voice: &str) -> Result<(PathBuf, PathBuf), String> {
    let (model_rel, config_rel) = match voice {
        "denis" => (
            "ru/ru_RU/denis/medium/ru_RU-denis-medium.onnx",
            "ru/ru_RU/denis/medium/ru_RU-denis-medium.onnx.json",
        ),
        "irina" => (
            "ru/ru_RU/irina/medium/ru_RU-irina-medium.onnx",
            "ru/ru_RU/irina/medium/ru_RU-irina-medium.onnx.json",
        ),
        other => return Err(format!("unknown voice: {other}")),
    };
    let root = home.join(".local/share/piper-voices");
    Ok((root.join(model_rel), root.join(config_rel)))
}

/// Run `command` with `input` written to its stdin, returning its captured
/// stdout/stderr/status once it exits.
///
/// Writing happens on a separate thread so a child that fills its stdout or
/// stderr pipe before finishing reading stdin (or vice versa) can't deadlock
/// against us.
fn run_with_stdin(mut command: Command, input: Vec<u8>) -> io::Result<Output> {
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = command.spawn()?;
    let mut stdin = child.stdin.take().expect("stdin was piped");
    let writer = std::thread::spawn(move || stdin.write_all(&input));
    let output = child.wait_with_output()?;
    let _ = writer.join();
    Ok(output)
}

/// Return zeroed S16LE mono PCM audio of the requested duration at
/// [`SAMPLE_RATE`].
fn silence(duration_seconds: f64) -> Vec<u8> {
    let num_samples = (f64::from(SAMPLE_RATE) * duration_seconds).round() as usize;
    vec![0u8; num_samples * 2]
}

/// Render `text` to raw S16LE mono PCM audio at [`SAMPLE_RATE`] via `piper`.
fn synthesize(model: &Path, config: &Path, text: &str) -> Result<Vec<u8>, String> {
    let mut command = Command::new("piper");
    command
        .arg("-m")
        .arg(model)
        .arg("-c")
        .arg(config)
        .arg("--output-raw");
    let output = run_with_stdin(command, text.as_bytes().to_vec())
        .map_err(|err| format!("failed to run piper: {err}"))?;
    if !output.status.success() {
        return Err(format!(
            "piper failed ({}): {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(output.stdout)
}

/// Encode raw S16LE mono PCM audio at [`SAMPLE_RATE`] to an mp3 file at
/// `output_path` via `ffmpeg`, overwriting it if it already exists.
fn encode_mp3(pcm: Vec<u8>, output_path: &Path) -> Result<(), String> {
    let mut command = Command::new("ffmpeg");
    command.args([
        "-y",
        "-hide_banner",
        "-loglevel",
        "error",
        "-f",
        "s16le",
        "-ar",
        &SAMPLE_RATE.to_string(),
        "-ac",
        "1",
        "-i",
        "-",
    ]);
    command.arg(output_path);

    let result =
        run_with_stdin(command, pcm).map_err(|err| format!("failed to run ffmpeg: {err}"))?;
    if !result.status.success() {
        return Err(format!(
            "ffmpeg failed ({}): {}",
            result.status,
            String::from_utf8_lossy(&result.stderr).trim()
        ));
    }
    Ok(())
}

pub fn run(id: u32) -> ExitCode {
    let Some(sentence) = sentences::run(id) else {
        eprintln!("No sentence with id {id}");
        return ExitCode::FAILURE;
    };

    let Some(home) = env::var_os("HOME").map(PathBuf::from) else {
        eprintln!("HOME environment variable is not set");
        return ExitCode::FAILURE;
    };

    let output_dir = Path::new(clause_audio::RU_MP3S_DIR)
        .join(clause_audio::VOL_1_PART_1_SUBDIR)
        .join(format!("{id:03}"));
    if let Err(err) = std::fs::create_dir_all(&output_dir) {
        eprintln!("failed to create {}: {err}", output_dir.display());
        return ExitCode::FAILURE;
    }

    let clauses = sentence.clauses();

    for voice in VOICES {
        let (model, config) = match voice_model_paths(&home, voice) {
            Ok(paths) => paths,
            Err(err) => {
                eprintln!("{err}");
                return ExitCode::FAILURE;
            }
        };
        for (field, path) in [("model", &model), ("model config", &config)] {
            if !path.exists() {
                eprintln!(
                    "{voice} {field} not found at {} (see src/mp3s.rs setup docs)",
                    path.display()
                );
                return ExitCode::FAILURE;
            }
        }

        for (index, clause) in clauses.iter().enumerate() {
            let clause_num = index + 1;
            let text = clause.text();

            let mut pcm = silence(LEAD_IN_SECONDS);
            match synthesize(&model, &config, &text) {
                Ok(audio) => pcm.extend(audio),
                Err(err) => {
                    eprintln!(
                        "failed to synthesize sentence {id} clause {clause_num} ({voice}): {err}"
                    );
                    return ExitCode::FAILURE;
                }
            }

            let output_path = Path::new(clause_audio::RU_MP3S_DIR).join(
                clause_audio::clause_mp3_relative_path(id, clause_num, voice),
            );
            if let Err(err) = encode_mp3(pcm, &output_path) {
                eprintln!("failed to encode {}: {err}", output_path.display());
                return ExitCode::FAILURE;
            }
            println!("wrote {}", output_path.display());
        }
    }

    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn voice_model_paths_denis_and_irina_differ() {
        let home = Path::new("/home/test");
        let denis = voice_model_paths(home, "denis").unwrap();
        let irina = voice_model_paths(home, "irina").unwrap();
        assert_ne!(denis, irina);
    }

    #[test]
    fn voice_model_paths_rejects_unknown_voice() {
        assert!(voice_model_paths(Path::new("/home/test"), "amy").is_err());
    }

    #[test]
    fn silence_scales_with_duration() {
        assert_eq!(silence(0.5).len(), silence(1.0).len() / 2);
    }

    #[test]
    fn run_reports_failure_for_unknown_sentence_id() {
        assert_eq!(run(999_999), ExitCode::FAILURE);
    }
}
