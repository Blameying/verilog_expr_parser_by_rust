extern crate lalrpop;
use std::{env, path::PathBuf};

fn main() {
    lalrpop::process_root().unwrap();
    let library_name = "espresso";
    let root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let library_dir = dunce::canonicalize(root.join("src")).unwrap();
    println!("cargo:rustc-link-lib=static={}", library_name);
    println!(
        "cargo:rustc-link-search=native={}",
        env::join_paths([library_dir]).unwrap().to_str().unwrap()
    );
}
