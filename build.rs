#[cfg(feature = "yaz0")]
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

#[cfg(feature = "yaz0")]
fn main() {
    build_zlib();
    let mut builder = cxx_build::bridge("src/yaz0.rs");
    builder
        .file("src/yaz0.cpp")
        .flag("-w")
        .flag_if_supported("-std=c++17")
        .include("src/include")
        .include("lib/nonstd")
        .include("lib/zlib-ng")
        .flag_if_supported("-static");
    if cfg!(windows) {
        builder.flag_if_supported("/std:c++17");
        // .flag_if_supported("/W4")
        // .flag_if_supported("/wd4244")
        // .flag_if_supported("/wd4127")
        // .flag_if_supported("/Zc:__cplusplus");
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
    println!("cargo:rustc-link-search=native=lib/zlib-ng");
    println!("cargo:rustc-link-lib=static=zlib");
}

#[cfg(not(feature = "yaz0"))]
fn main() {}
