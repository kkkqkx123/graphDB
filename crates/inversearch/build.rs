fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Generate proto code for gRPC service
    // Proto code will be generated but only used when "service" feature is enabled

    println!("cargo:rerun-if-changed=proto/inversearch.proto");

    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .out_dir(std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()))
        .compile_protos(&["proto/inversearch.proto"], &["proto/"])?;

    Ok(())
}
