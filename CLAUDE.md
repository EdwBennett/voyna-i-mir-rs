# Project instructions

## Target platform
- Primary target: Fedora KDE (x86_64).
- This project contains Rust and Python.
- Do not assume Ubuntu-specific package names, paths, or desktop behavior.
- Cloud-based tests do not validate Fedora-specific, KDE/Plasma, Wayland,
  SELinux, systemd, hardware, or GUI behavior.

## Rust workflow
- Type check: `cargo check --workspace --all-targets`
- Format check: `cargo fmt --check`
- Lint: `cargo clippy --workspace --all-targets -- -D warnings`
- Test: `cargo test --workspace`
- Ask before adding dependencies or changing public APIs.

## Python workflow
- Scripts in `python/` are short and ad hoc: copied into an interactive
  `python3` session and hand-edited per use (e.g. swapping the `words`
  key), not run standalone. No dependency/environment file — assume
  the interpreter already has what's needed (e.g. PyYAML).
- Format: `ruff format python/`
- Lint: `ruff check python/`
- No test suite — verify by inspecting the generated output file.
- Do not introduce undeclared Python dependencies.

## Completion criteria
- State the files changed.
- State every validation command actually run and its result.
- Clearly identify tests that could not be run.
- Do not say a change is fully verified if only cloud/Ubuntu testing occurred.

## Git policy
- Do not run `git commit`, `git push`, `git rebase`, `git reset`,
  or change branches unless explicitly requested.
- I review diffs and run final `cargo run ...` validation locally in Konsole.
