use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(
        env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR must be set by Cargo"),
    );
    let ui_dist = manifest_dir.join("ui/dist");

    let embed_dir = if ui_dist.is_dir() {
        println!("cargo:rerun-if-changed={}", ui_dist.display());
        ui_dist
    } else {
        let fallback_dir =
            PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR must be set by Cargo"))
                .join("ui-dist");
        fs::create_dir_all(&fallback_dir).expect("failed to create fallback UI directory");
        fs::write(
            fallback_dir.join("index.html"),
            "<!doctype html><title>Event Gateway</title>",
        )
        .expect("failed to create fallback UI");
        fallback_dir
    };

    println!(
        "cargo:rustc-env=EVENT_GATEWAY_UI_DIR={}",
        embed_dir.display()
    );
}
