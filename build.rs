use std::process::{Command, Stdio};
use rustc_version::version_meta;

fn main() {
    let meta = version_meta().unwrap();
    
    let toolchain = format!("{:#?}{}", meta.channel, match meta.commit_date {
        Some(date) => format!("-{}", date),
        None => String::new(),
    }).to_lowercase();
    
    if !install_toolchain(&toolchain) {
        panic!("Failed to install rust toolchain {}", toolchain);
    }
}

fn install_toolchain(toolchain: &str) -> bool {
    let output = Command::new("rustup")
        .args([
            "install",
            toolchain,
        ])
        .stderr(Stdio::inherit())
        .output()
        .unwrap();
    if !output.status.success() {
        return false;
    }
    
    let output = Command::new("rustup")
        .args([
            "run",
            toolchain,
            "--",
            "rustup",
            "component",
            "add",
            "rust-src",
        ])
        .stderr(Stdio::inherit())
        .output()
        .unwrap();
    
    output.status.success()
}