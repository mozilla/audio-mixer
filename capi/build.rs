fn main() {
    let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    println!("cargo:rerun-if-changed=src/lib.rs");
    cbindgen::generate(&crate_dir)
        .expect("Could not generate header")
        .write_to_file("include/audio_mixer.h");
}
