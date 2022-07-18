fn build_zlib() {
    std::process::Command::new("cmake")
        .current_dir("lib/zlib-ng")
        .arg(".")
        .output()
        .unwrap();
    std::process::Command::new("cmake")
        .current_dir("lib/zlib-ng")
        .arg("--build")
        .arg(".")
        .output()
        .unwrap();
}

fn main() {
    #[cfg(feature = "yaz0")]
    build_zlib();

    let bridge_files = [
        #[cfg(feature = "yaz0")]
        "src/yaz0.rs",
        // #[cfg(feature = "byml")]
        // "src/byml.rs",
    ];
    let source_files = [
        #[cfg(feature = "yaz0")]
        "src/yaz0.cpp",
        // #[cfg(feature = "byml")]
        // "src/byml.cpp",
    ];

    let mut builder = cxx_build::bridges(bridge_files);
    builder
        .files(source_files)
        .compiler("clang++")
        .flag("-w")
        .flag_if_supported("-std=c++17")
        .include("src/include")
        .include("lib/abseil")
        .include("lib/EasyIterator/include")
        .include("lib/libyaml")
        .include("lib/nonstd")
        .include("lib/ordered-map/include")
        .include("lib/pybind11")
        .include("lib/rapidyaml/src")
        .include("lib/rapidyaml/ext/c4core/src")
        .include("lib/rapidyaml/ext/c4core/ext")
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

    #[cfg(feature = "yaz0")]
    {
        println!("cargo:rerun-if-changed=src/yaz0.rs");
        println!("cargo:rerun-if-changed=src/yaz0.cpp");
        println!("cargo:rerun-if-changed=src/include/oead/yaz0.h");
        println!("cargo:rustc-link-search=native=lib/zlib-ng");
        println!("cargo:rustc-link-lib=static=zlib");
    }
    #[cfg(feature = "byml")]
    {
        println!("cargo:rerun-if-changed=src/byml.rs");
        println!("cargo:rerun-if-changed=src/byml.cpp");
        println!("cargo:rerun-if-changed=src/include/oead/byml.h");
    }
}
