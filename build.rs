fn main() {
    cxx_build::bridge("src/lib.rs")
        .flag("-w")
        .files(
            [
                "src/byml/byml.cc",
                "src/sarc/sarc.cc",
                "src/types/types.cc",
                "src/yaz0/yaz0.cc",
            ]
            .iter(),
        )
        .include("include/oead/src/include")
        .include("include/oead/lib/abseil")
        .include("include/oead/lib/EasyIterator/include")
        .include("include/oead/lib/libyaml")
        .include("include/oead/lib/nonstd")
        .include("include/oead/lib/ordered-map")
        .include("include/oead/lib/pybind11")
        .include("include/oead/lib/rapidyaml")
        .include("include/oead/lib/zlib-ng")
        .flag_if_supported("-std=c++17")
        .flag_if_supported("-static")
        .compile("roead");

    for file in glob::glob("include/oead/build/**/*.a")
        .unwrap()
        .flat_map(|f| f.ok())
    {
        if file.to_str().unwrap().contains("subprojects") {
            continue;
        }
        println!(
            "cargo:rustc-link-search=native={}",
            file.parent().unwrap().to_str().unwrap()
        );
        println!(
            "cargo:rustc-link-lib=static={}",
            &file.file_stem().unwrap().to_str().unwrap()[3..]
        );
    }
    println!("cargo:rerun-if-changed=src/lib.rs");
    println!("cargo:rerun-if-changed=src/roead.cc");
    println!("cargo:rerun-if-changed=include/roead.h");
}
