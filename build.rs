use std::process::Command;

fn main() {
    // Run Vite in web directory before building
    println!("cargo:rerun-if-changed=web/src");
    println!("cargo:rerun-if-changed=web/style");
    println!("cargo:rerun-if-changed=web/scripts");
    println!("cargo:rerun-if-changed=web/package.json");
    println!("cargo:rerun-if-changed=web/vite.config.ts");

    let output = Command::new("pnpm")
        .args(["run", "build:icons"])
        .current_dir("web")
        .output()
        .expect("Failed to run vite (build-icons)");
    if !output.status.success() {
        panic!(
            "Vite (build-icons) failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let output = Command::new("pnpm")
        .args(["run", "build"])
        .current_dir("web")
        .output()
        .expect("Failed to run vite (build)");
    if !output.status.success() {
        panic!(
            "Vite (build) failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    println!("cargo:warning=Assets bundled with Vite successfully");
}
