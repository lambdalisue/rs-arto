use std::process::Command;

fn main() {
    // Run Vite in web directory before building
    println!("cargo:rerun-if-changed=web/src/main.css");
    println!("cargo:rerun-if-changed=web/src/main.js");
    println!("cargo:rerun-if-changed=web/package.json");
    println!("cargo:rerun-if-changed=web/vite.config.ts");

    let output = Command::new("pnpm")
        .args(["run", "build"])
        .current_dir("web")
        .output()
        .expect("Failed to run vite");

    if !output.status.success() {
        panic!("Vite failed: {}", String::from_utf8_lossy(&output.stderr));
    }
    println!("cargo:warning=Assets bundled with Vite successfully");
}
