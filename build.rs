use std::{collections::BTreeSet, path::PathBuf, process::Command};

fn main() {
    println!("cargo:rerun-if-changed=src/lib.rs");
    let root = std::env::var("CARGO_MANIFEST_DIR")
        .unwrap()
        .replace('\\', "/");
    for file in glob::glob(&format!("{}/src/**/*.cc", &root))
        .unwrap()
        .filter_map(|f| f.ok())
        .chain(
            glob::glob(&format!("{}/include/*.h", &root))
                .unwrap()
                .filter_map(|f| f.ok()),
        )
    {
        println!(
            "cargo:rerun-if-changed={}",
            file.strip_prefix(PathBuf::from(&root))
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/")
        );
    }
    println!("cargo:rerun-if-changed=Cargo.toml");
    std::fs::create_dir("include/oead/build").unwrap_or(());
    Command::new("cmake")
        .current_dir("include/oead/build")
        .args(&["../"])
        .output()
        .expect("Failed to run CMake");
    let target_os = std::env::var("CARGO_CFG_TARGET_OS");
    match target_os.as_ref().map(|x| &**x) {
        Ok("linux") => {
            Command::new("make")
                .current_dir("include/oead/build")
                .output()
                .expect("Failed to run CMake");
            cxx_build::bridge("src/lib.rs")
                .flag("-w")
                .files(
                    [
                        "src/aamp/aamp.cc",
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
                .include("include/oead/lib/ordered-map/include")
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
        }
        Ok("windows") => {
            Command::new("cmake")
                .current_dir("include/oead/build")
                .args(&["--build", ".", "--config", "release"])
                .output()
                .expect("Failed to run CMake");
            cxx_build::bridge("src/lib.rs")
                .flag("-w")
                .files(
                    [
                        "src/aamp/aamp.cc",
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
                .include("include/oead/lib/ordered-map/include")
                .include("include/oead/lib/pybind11")
                .include("include/oead/lib/rapidyaml")
                .include("include/oead/lib/zlib-ng")
                .flag_if_supported("-std=c++17")
                .flag_if_supported("/std:c++17")
                .flag_if_supported("-static")
                .compile("roead");
            let mut files: Vec<PathBuf> = glob::glob("include/oead/build/**/*.lib")
                .unwrap()
                .flat_map(|f| f.ok())
                .collect();
            files.sort_by_key(|f| f.ancestors().count());
            files.reverse();
            let (search_locs, libs): (BTreeSet<String>, BTreeSet<String>) = files
                .into_iter()
                .filter_map(|file| -> Option<(String, String)> {
                    let name = file.to_str().unwrap();
                    if name.contains("subprojects")
                        || name.contains("Debug")
                        || name.contains("target")
                    {
                        None
                    } else {
                        Some((
                            file.parent().unwrap().to_str().unwrap().to_string(),
                            file.file_stem()
                                .unwrap()
                                .to_str()
                                .unwrap()
                                .strip_prefix("lib")
                                .unwrap_or_else(|| file.file_stem().unwrap().to_str().unwrap())
                                .to_owned(),
                        ))
                    }
                })
                .unzip();
            search_locs
                .iter()
                .for_each(|l| println!("cargo:rustc-link-search=native={}", l));
            libs.iter()
                .for_each(|l| println!("cargo:rustc-link-lib=static={}", l));
        }
        _ => panic!("Not a supported target"),
    }
}
