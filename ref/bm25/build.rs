fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "service")]
    {
        tonic_build::configure()
            .build_server(true)
            .build_client(true)
            .compile_protos(&["proto/bm25.proto"], &["proto/"])?;
    }

    Ok(())
}
