use std::{env, path::Path};

use rustc_version::{version_meta, Channel};

#[cfg(feature = "yaz0")]
fn build_zlib() {
    let target = env::var("TARGET").unwrap();
    let mut cmake = std::process::Command::new("cmake");
    cmake.current_dir("lib/zlib-ng");
    if target.contains("aarch64-apple-darwin") {
        cmake.arg("-DCMAKE_OSX_ARCHITECTURES=arm64");
    } else if target.contains("x86_64-apple-darwin") {
        cmake.arg("-DCMAKE_OSX_ARCHITECTURES=x86_64");
    } else {
        //Not OSX
    }
    cmake
        .arg(".")
        .output()
        .expect("Failed to build zlib. Is CMake installed?");
    std::process::Command::new("cmake")
        .current_dir("lib/zlib-ng")
        .arg("--build")
        .arg(".")
        .output()
        .expect("Failed to build zlib");
}

#[cfg(feature = "yaz0")]
fn build_yaz0() {
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
        builder
            .flag_if_supported("/std:c++17")
            .flag_if_supported("/W4")
            .flag_if_supported("/wd4244")
            .flag_if_supported("/wd4127")
            .flag_if_supported("/Zc:__cplusplus");
        println!("cargo:rustc-link-search=native=lib/zlib-ng/Debug");
        println!("cargo:rustc-link-search=native=lib/zlib-ng/Release");
        println!("cargo:rustc-link-lib=static=zlibd");
    } else {
        builder
            .flag_if_supported("-fcolor-diagnostics")
            .flag_if_supported("-Wall")
            .flag_if_supported("-Wextra")
            .flag_if_supported("-fno-plt");
        println!("cargo:rustc-link-lib=static=zlib");
    }
    builder.compile("roead");
    println!("cargo:rerun-if-changed=src/include/oead");
    println!("cargo:rerun-if-changed=src/yaz0.rs");
    println!("cargo:rerun-if-changed=src/yaz0.cpp");
    println!("cargo:rerun-if-changed=src/include/oead/yaz0.h");
    let dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    println!(
        "cargo:rustc-link-search=native={}",
        Path::new(&dir).join("lib/zlib-ng").display()
    );
}

fn main() {
    // Set cfg flags depending on release channel
    let channel = match version_meta().unwrap().channel {
        Channel::Stable => "CHANNEL_STABLE",
        Channel::Beta => "CHANNEL_BETA",
        Channel::Nightly => "CHANNEL_NIGHTLY",
        Channel::Dev => "CHANNEL_DEV",
    };
    println!("cargo:rustc-cfg={}", channel);
    println!("cargo::rustc-check-cfg=cfg({})", channel);
    #[cfg(feature = "yaz0")]
    build_yaz0();
}
