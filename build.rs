use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::Command;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");

    let git_version_path = Path::new(&manifest_dir).join("src/git-version.txt");

    let status_output = Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .expect("Failed to execute git status command");
    let in_git_repo = status_output.status.success();
    let git_vers_exists = git_version_path.exists();

    assert!(
        in_git_repo || git_vers_exists,
        "Panicking because there's no .git and {} otherwise cannot be determined",
        git_version_path.display()
    );

    if git_vers_exists && !in_git_repo {
        println!(
            "cargo:warning=Because a .git could not be found, the existing {} will be used",
            git_version_path.display()
        );
        return;
    }

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

    let mut file =
        fs::File::create(&git_version_path).expect("Failed to create git commit hash file");
    file.write_all(git_version.as_bytes())
        .expect("Failed to write git commit hash to file");

    println!(
        "cargo:warning=Git commit hash written to {}",
        git_version_path.display()
    );

    let manifest_path = Path::new(&manifest_dir).join("Cargo.toml");
    let manifest = std::fs::read_to_string(&manifest_path).unwrap();
    let manifest: toml::Value = toml::from_str(&manifest).unwrap();
    let crate_version = manifest["package"]["version"]
        .as_str()
        .expect("missing crate version");
    let long_version_path = Path::new(&manifest_dir).join("src/long-version.txt");

    let mut file =
        fs::File::create(&long_version_path).expect("Failed to create long version file");
    write!(file, "{crate_version} {git_version}").expect("Failed to write long version to file");

    println!(
        "cargo:warning=Long version written to {}",
        long_version_path.display()
    );
}
