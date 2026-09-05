Formatting is clean. `cargo check`/`clippy` couldn't run workspace-wide (pre-existing toolchain gap, not related to this file) — details below. Here's the manual review of `src/mp3s.rs`.

## Summary

The file is solid, idiomatic Rust overall: `ExitCode` returns instead of `process::exit`, `?`/`.map_err` for error propagation, `&'static str` for the static voice list (no needless allocation), doc comments on every public item, and a `#[cfg(test)]` module at the bottom — all standard per the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/). A couple of genuine (minor) issues and a few stylistic nits.

## Findings

**1. `run_with_stdin` silently discards a real I/O error (mp3s.rs:64-68)**
```rust
let writer = std::thread::spawn(move || stdin.write_all(&input));
let output = child.wait_with_output()?;
let _ = writer.join();
```
`writer.join()` returns `thread::Result<io::Result<()>>` — both the `JoinHandle` result *and* the inner `write_all` result are dropped. A `BrokenPipe` from the child exiting early is expected and fine to ignore, but any other write failure (e.g. `ENOSPC`, a killed child) is swallowed too, and the caller only ever sees the *read* side's exit status/stderr. Since `synthesize`/`encode_mp3` already report `output.status`, a non-BrokenPipe write error would currently surface as a possibly-confusing "process failed" with unhelpful stderr rather than the real cause. Consider matching on the inner `io::Result` and logging (or including in the error) anything that isn't `ErrorKind::BrokenPipe`.

**2. Duplicated "resolve voices, then act" scaffolding (mp3s.rs:217-249)**
`run` and `run_all` both do:
```rust
let voices = match resolve_voices_from_env() {
    Ok(voices) => voices,
    Err(code) => return code,
};
```
then differ only in what they do with `voices`. Minor duplication (4 lines) — could be a shared helper, but it's small enough that this is a judgment call rather than a real problem.

**3. `voice_model_paths` returns an unnamed tuple `(PathBuf, PathBuf)` (mp3s.rs:36)**
Since the very next thing `resolve_voices` does is destructure it straight into the named fields of `VoicePaths`, a tuple is defensible glue here, but a small named return (or building `VoicePaths` directly inside `voice_model_paths`) would read slightly more self-documenting at the call site. Low priority.

**4. `Path::new(clause_audio::RU_MP3S_DIR)` is constructed twice in `render_sentence`** (mp3s.rs:170, 194) — once for `output_dir`, once for `output_path`. `Path::new` on a `&'static str` is free (no allocation, just a reinterpret), so this is not a performance issue, just a very minor repetition; not worth changing.

**Not a problem (verified):** the `command.args([...])` call in `encode_mp3` (mp3s.rs:103-116) mixes `&str` literals with `&SAMPLE_RATE.to_string()` (a `&String`) inside one array literal. I was initially suspicious this wouldn't type-check against `Command::args<I: IntoIterator<Item = S>, S: AsRef<OsStr>>`, but confirmed with a standalone `rustc` compile that array-literal LUB coercion resolves `&String` → `&str` here, so it's valid and is in fact the normal idiom for this situation.

## What I could and couldn't validate

- `cargo fmt --check -- src/mp3s.rs` — ran clean.
- `cargo check --workspace --all-targets` / `cargo clippy --workspace --all-targets -- -D warnings` — **could not run**: this sandbox's `rustc` is 1.94.1, but `eframe`/`egui` 0.36.1 (an existing workspace dependency, unrelated to this file) require rustc 1.95. This is a pre-existing environment/toolchain gap, not something introduced by `mp3s.rs`, and per CLAUDE.md I'm not treating this as verified — you'll want to run `cargo check`/`clippy`/`test` locally in Konsole as usual.
- `cargo test --workspace` — not run, same toolchain blocker.
- No functional changes were made; this was a review only, so nothing needs building/running to "work" — but the clippy/check gap means I can't independently confirm clippy is silent on the two flagged spots beyond my manual reading.

No files were changed. Want me to apply fixes for finding #1 (the swallowed write error) and #2 (dedup the resolve-then-act scaffolding)?
