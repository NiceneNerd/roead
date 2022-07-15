fn main() {
    std::process::Command::new("cmake")
        .current_dir("lib/zlib-ng")
        .arg(".")
        .output()
        .unwrap();
    let bridge_files = [
        #[cfg(feature = "yaz0")]
        "src/yaz0.rs",
    ];
    let source_files = [
        #[cfg(feature = "yaz0")]
        "src/yaz0.cpp",
    ];
    let mut builder = cxx_build::bridges(bridge_files);
    builder
        .files(source_files)
        .flag("-w")
        .flag_if_supported("-std=c++17")
        .include("src/include")
        .include("lib/abseil")
        .include("lib/EasyIterator/include")
        .include("lib/libyaml")
        .include("lib/nonstd")
        .include("lib/ordered-map/include")
        .include("lib/pybind11")
        .include("lib/rapidyaml")
        .include("lib/zlib-ng")
        .flag_if_supported("-static");
    if cfg!(windows) {
        builder
            .flag_if_supported("/W4")
            .flag_if_supported("/wd4244")
            .flag_if_supported("/wd4127")
            .flag_if_supported("/Zc:__cplusplus");
    } else {
        builder
            .flag_if_supported("-fcolor-diagnostics")
            .flag_if_supported("-Wall")
            .flag_if_supported("-Wextra")
            .flag_if_supported("-fno-plt");
    }
    builder.compile("roead");
    println!("cargo:rerun-if-changed=src/include/oead");
    println!("cargo:rerun-if-changed=src/yaz0.rs");
    println!("cargo:rerun-if-changed=src/yaz0.cpp");
    println!("cargo:rerun-if-changed=src/include/oead/yaz0.h");
}
