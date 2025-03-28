use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let app_names = vec!["code_editor"]; // Add more apps here as needed

    for app in &app_names {
        println!("cargo:rerun-if-changed=../apps/{app}/src");
        println!("Building Collab Hub app: {app}");

        // Run `dx build --release` inside the app directory
        let status = Command::new("dx")
            .args(["build", "--release"])
            .current_dir(format!("../apps/{app}"))
            .status()
            .expect("Failed to run dx build");

        if !status.success() {
            panic!("Build failed for app: {app}");
        }

        // From: target/dx/code_editor/release
        let from_str = &format!("../target/dx/{app}/release/web/public");
        let from = Path::new(from_str);
        // To: backend/dist/code_editor
        let to_str = &format!("../dist/{app}");
        let to = Path::new(to_str);

        if to.exists() {
            fs::remove_dir_all(&to).expect("Failed to clean previous dist folder");
        }

        copy_dir_filtered(from, to).expect("Failed to copy built files");
    }
}

/// Copy files from `from` to `to`, skipping unwanted directories
fn copy_dir_filtered(from: &Path, to: &Path) -> std::io::Result<()> {
    for entry in walkdir::WalkDir::new(from) {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            continue;
        }

        // Get relative path
        let relative_path = path.strip_prefix(from).unwrap();
        let target_path = to.join(relative_path);

        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::copy(path, &target_path)?;
    }

    Ok(())
}
