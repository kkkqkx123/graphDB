fn main() {
    // Enable Vulkan support when llama_cpp feature is enabled
    #[cfg(feature = "llama_cpp")]
    {
        println!("cargo:rustc-env=CMAKE_ARGS=-DGGML_VULKAN=on");
        println!("cargo:rerun-if-changed=build.rs");
    }
}
