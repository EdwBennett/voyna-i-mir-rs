# Война и мир (voyna-i-mir-rs)

A Russian-language study tool built around excerpts from *War and Peace*.
Each sentence is stored with a word-by-word gloss (Russian, romanization,
and English meaning), which can be printed as JSON or explored interactively
in a desktop GUI.

## Usage

```sh
# Print the sentence with id 1 as JSON
cargo run -- sentence 1

# Open an egui window to explore the sentence with id 1
cargo run -- display 1
```

In the GUI, click a word (or use the left/right arrow keys and space) to
reveal its romanization and English gloss.

## Live demo

A static, browser-based version is published via GitHub Pages:

<https://edwbennett.github.io/voyna-i-mir-rs/index.html?id=1>

Change the `id` query parameter to view a different sentence.

## Data

Excerpt text and glosses live as YAML under [src/excerpts/](src/excerpts/),
keyed by volume/part in [voyna-i-mir.yaml](src/excerpts/voyna-i-mir.yaml).
The [python/](python/) directory holds a small ad hoc script,
[print_words.py](python/print_words.py), used to dump a YAML excerpt file to
plain text for review.

## Development

```sh
cargo check --workspace --all-targets
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```
