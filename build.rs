use std::process::Command;

fn main() {
    // Run Vite in renderer directory before building
    println!("cargo:rerun-if-changed=renderer/src");
    println!("cargo:rerun-if-changed=renderer/style");
    println!("cargo:rerun-if-changed=renderer/scripts");
    println!("cargo:rerun-if-changed=renderer/icons.json");
    println!("cargo:rerun-if-changed=renderer/package.json");
    println!("cargo:rerun-if-changed=renderer/vite.config.ts");
    println!("cargo:rerun-if-changed=assets/welcome.md");
    println!("cargo:rerun-if-changed=assets/arto-header-welcome.png");

    let output = Command::new("pnpm")
        .args(["run", "build"])
        .current_dir("renderer")
        .output()
        .expect("Failed to run vite (build)");
    if !output.status.success() {
        panic!(
            "Vite (build) failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
