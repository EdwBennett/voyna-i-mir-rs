//! Filesystem/URL layout for per-clause voice mp3s, shared between the
//! `mp3s` CLI (which generates them, native-only, see `src/mp3s.rs`) and
//! `page2` (which plays them, both native and wasm).

/// Absolute path to the `src/ru-mp3s` directory, resolved at compile time
/// so it doesn't depend on `cargo run`'s cwd. Native-only: on wasm, trunk's
/// `copy-dir` in index.html serves the same tree at the `ru-mp3s/` URL
/// instead.
#[cfg(not(target_arch = "wasm32"))]
pub const RU_MP3S_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src/ru-mp3s");

/// Subdirectory of `ru-mp3s/` that `Volume_1_Part_1.yaml`'s clause mp3s
/// live in.
pub const VOL_1_PART_1_SUBDIR: &str = "vol-1-part-1";

/// Path to `sentence_id`'s `clause_num`-th clause (1-based) mp3 in `voice`,
/// relative to `RU_MP3S_DIR` (native) or the deployed `ru-mp3s/` URL (wasm).
pub fn clause_mp3_relative_path(sentence_id: u32, clause_num: usize, voice: &str) -> String {
    format!("{VOL_1_PART_1_SUBDIR}/{sentence_id:03}/{clause_num:02}_{voice}.mp3")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clause_mp3_relative_path_pads_id_and_clause_number() {
        assert_eq!(
            clause_mp3_relative_path(1, 2, "denis"),
            "vol-1-part-1/001/02_denis.mp3"
        );
        assert_eq!(
            clause_mp3_relative_path(42, 11, "irina"),
            "vol-1-part-1/042/11_irina.mp3"
        );
    }
}
