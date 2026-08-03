fn main() {
    println!("cargo:rerun-if-changed=wrapper.h");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg("-I../vendor/c-api/protocol/")
        .allowlist_function("em.*")
        .allowlist_type("em.*")
        .allowlist_var("EM.*")
        .allowlist_var("EMBER_.*")
        .generate()
        .unwrap();

    bindings.write_to_file("src/bindings.rs").unwrap();
}
