fn main() {
    cxx_build::bridge("src/yaz0.rs") // returns a cc::Build
        .file("src/yaz0.cpp")
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
        .flag_if_supported("-static")
        .compile("roead");
    println!("cargo:rerun-if-changed=src/yaz0.rs");
    println!("cargo:rerun-if-changed=src/yaz0.cpp");
    println!("cargo:rerun-if-changed=src/include/oead/yaz0.h");
}
