use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

fn main() {
    let css_version = hash_files(&["static/font.css", "static/style.css"]);
    let js_version = hash_files(&["static/js/commit.js"]);

    println!("cargo:rustc-env=CSS_VERSION={css_version}");
    println!("cargo:rustc-env=JS_VERSION={js_version}");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=static/font.css");
    println!("cargo:rerun-if-changed=static/style.css");
    println!("cargo:rerun-if-changed=static/js/commit.js");
}

fn hash_files(paths: &[&str]) -> String {
    let mut hasher = Sha256::new();

    for path in paths {
        let path = Path::new(path);
        hasher.update(path.to_string_lossy().as_bytes());
        hasher.update([0]);
        hasher.update(
            fs::read(path).unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display())),
        );
        hasher.update([0xff]);
    }

    let digest = hasher.finalize();
    format!("{:x}", digest)[..12].to_string()
}
