use std::env;
use std::path::Path;
use std::fs;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let target = env::var("TARGET").unwrap();

    let cbc_path = match target.as_str() {
        "x86_64-pc-windows-msvc" => "binaries/windows/cbc.exe",
        "x86_64-unknown-linux-gnu" => "binaries/linux/cbc",
        "x86_64-apple-darwin" => "binaries/macos/cbc",
        _ => panic!("Unsupported target: {}", target),
    };

    fs::copy(cbc_path, Path::new(&out_dir).join("cbc")).unwrap();
    println!("cargo:rerun-if-changed=binaries");
}