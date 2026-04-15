fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Generate proto code for gRPC service
    // Only compile proto when "grpc" feature is enabled

    #[cfg(feature = "grpc")]
    {
        println!("cargo:rerun-if-changed=proto/graphdb.proto");

        tonic_build::configure()
            .build_server(true)
            .build_client(true)
            .out_dir(std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()))
            .compile_protos(&["proto/graphdb.proto"], &["proto/"])?;
    }

    Ok(())
}
