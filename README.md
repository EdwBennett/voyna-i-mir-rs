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

A static, browser-based version is published via GitHub Pages. The `page`
query parameter selects which page to show, and `id` selects the sentence:

- [Word-by-word gloss page](https://edwbennett.github.io/voyna-i-mir-rs/?page=1&id=1)
- [Clause audio page](https://edwbennett.github.io/voyna-i-mir-rs/?page=2&id=1)

Change the `id` query parameter to view a different sentence.

### Clause audio page controls

- Click a clause to select it and start looping the selected voice;
  clicking the selected clause again deselects it and stops playback.
- Left/Right arrow keys select the previous/next clause, wrapping around
  the ends of the sentence, without starting playback.
- `I`/`D` select the Irina/Denis voice and start it looping the selected
  (or first) clause.
- `A` alternates the voice every loop.
- Space toggles playback: starts it if idle, or lets the current loop
  finish and then stops if playing.
- Tapping the "Voice: Irina / Denis" row at the top cycles through the
  three voice choices (Irina, Denis, or both highlighted for alternate) -
  the touch equivalent of `I`/`D`/`A`.
- Ctrl+Right/Ctrl+Left move to the next/previous chapter, wrapping around
  the ends of the excerpt list. The voice selection carries over; clause
  selection and playback reset. The `<<`/`>>` labels between the chapter
  heading and the Voice text are the touch equivalent.
- Ctrl+W closes the window (native only).

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
