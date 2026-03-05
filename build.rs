fn main() {
    #[cfg(feature = "c_api")]
    {
        println!("cargo:rerun-if-changed=src/api/embedded/c_api");
        println!("cargo:rerun-if-changed=cbindgen.toml");

        cbindgen::Builder::new()
            .with_crate(".")
            .with_language(cbindgen::Language::C)
            .with_pragma_once(true)
            .with_include_guard("GRAPHDB_H")
            .with_header(
                "GraphDB C API\n\
                 \n\
                 GraphDB C API 头文件\n\
                 提供 GraphDB 的 C 语言接口\n\
                 \n\
                 版本: 0.1.0\n\
                 许可: Apache-2.0\n\
                 \n\
                 更多信息请访问: https://github.com/kkkqkx123/graphDB",
            )
            .generate()
            .expect("Unable to generate bindings")
            .write_to_file("include/graphdb.h");
    }
}
