use std::{env, error::Error, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    if env::var("TARGET")? == "wasm32-unknown-unknown" {
        let pre_built_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?).join("pre-built");
        println!("cargo:rustc-link-lib=static=calloca");
        println!("cargo:rustc-link-search=native={}", pre_built_dir.display());
    }

    Ok(())
}
