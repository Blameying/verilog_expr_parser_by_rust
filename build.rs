extern crate cc;
extern crate lalrpop;
use std::{env, fs, path::PathBuf};

fn main() {
    lalrpop::process_root().unwrap();
    let library_name = "espresso";
    let root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let library_dir = dunce::canonicalize(root.join("src").join("espresso-src")).unwrap();

    let c_files: Vec<PathBuf> = fs::read_dir(&library_dir)
        .unwrap()
        .filter_map(Result::ok)
        .filter(|f| {
            f.file_type().unwrap().is_file() && f.file_name().to_string_lossy().ends_with(".c")
        })
        .map(|f| f.path())
        .collect();
    cc::Build::new()
        .files(c_files)
        .include(library_dir)
        .compile("libespresso.a");

    println!("cargo:rustc-link-lib=static={}", library_name);
}
