use std::error::Error;

fn main() -> std::result::Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=wrapper.h");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg("-Ivendor/c-api/protocol/")
        .allowlist_function("em.*")
        .allowlist_type("em.*")
        .allowlist_var("EM.*")
        .allowlist_var("EMBER_.*")
        .generate()?;

    bindings.write_to_file("src/ffi.rs")?;
    Ok(())
}
