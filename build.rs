fn main() -> Result<(), Box<dyn std::error::Error>> {
    // cbindgen will be invoked separately to generate C headers
    println!("cargo:rerun-if-changed=src/lib.rs");
    Ok(())
}