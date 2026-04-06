#[cfg(feature = "service")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(&["proto/bm25.proto"], &["proto/"])?;

    Ok(())
}

#[cfg(not(feature = "service"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}
