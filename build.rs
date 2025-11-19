use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::Command;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");

    let status_output = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .expect("Failed to execute git status command");
    let in_git_repo = status_output.status.success();

    assert!(in_git_repo, "Panicking because there's no .git",);

    let git2 = vergen_git2::Git2Builder::default()
        .sha(false)
        .dirty(true)
        .build()
        .unwrap();
    vergen_git2::Emitter::default()
        .add_instructions(&git2)
        .unwrap()
        .emit()
        .unwrap();

    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .expect("Failed to execute git command");

    let git_sha = String::from_utf8(output.stdout)
        .expect("Invalid UTF-8 output from git")
        .trim()
        .to_string();

    let status_str =
        String::from_utf8(status_output.stdout).expect("Invalid UTF-8 output from git status");

    let is_dirty = !status_str.trim().is_empty();

    let mut git_version = git_sha.clone();
    if is_dirty {
        git_version.push('*');
        println!("cargo:warning=Repository is dirty, adding * to commit hash");
    }

    println!("cargo:rustc-env=PUNCH_CLOCK_GIT_REVISION={git_version}");
    println!("cargo:warning=Git commit revision exported to PUNCH_CLOCK_GIT_REVISION",);

    let manifest_path = Path::new(&manifest_dir).join("Cargo.toml");
    let manifest = std::fs::read_to_string(&manifest_path).unwrap();
    let manifest: toml::Value = toml::from_str(&manifest).unwrap();
    let crate_version = manifest["package"]["version"]
        .as_str()
        .expect("missing crate version");

    let long_version = format!("{crate_version} {git_version}");
    println!("cargo:rustc-env=PUNCH_CLOCK_LONG_VERSION={long_version}");
    println!("cargo:warning=Long version {long_version:?} exported to PUNCH_CLOCK_GIT_REVISION",);
}
