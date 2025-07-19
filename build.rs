use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    // Add build metadata
    let git_hash = get_git_hash();
    let build_date = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC");

    println!("cargo:rustc-env=GIT_HASH={git_hash}");
    println!("cargo:rustc-env=BUILD_DATE={build_date}");
    println!(
        "cargo:rustc-env=BUILD_TARGET={}",
        env::var("TARGET").unwrap_or_default()
    );

    // Rerun if git state changes
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs/");

    // Generate completion files if needed
    generate_shell_completions();

    // Set version information
    let version = env::var("CARGO_PKG_VERSION").unwrap();
    println!("cargo:rustc-env=TSRC_VERSION={version}");

    // Enable static linking for specific targets
    let target = env::var("TARGET").unwrap();
    if target.contains("musl") {
        println!("cargo:rustc-link-arg=-static");
    }

    if target.contains("windows") {
        // Enable static CRT linking for Windows
        println!("cargo:rustc-link-arg=/DEFAULTLIB:libcmt");
    }
}

fn get_git_hash() -> String {
    if let Ok(output) = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
    {
        if output.status.success() {
            return String::from_utf8_lossy(&output.stdout).trim().to_string();
        }
    }
    "unknown".to_string()
}

fn generate_shell_completions() {
    // This would generate shell completions during build
    // For now, we'll just create the directory structure
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let completions_dir = Path::new(&out_dir).join("completions");

    if !completions_dir.exists() {
        fs::create_dir_all(&completions_dir).ok();
    }

    // In a real implementation, this would use clap's completion generation
    // generate_completion_files(&completions_dir);
}
