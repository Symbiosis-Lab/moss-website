fn main() {
    // Track CSS file changes to trigger recompilation when assets change
    // 
    // CRITICAL: The CSS is embedded at compile time via include_str! in compile.rs.
    // While include_str! should automatically track dependencies, Cargo's behavior
    // is inconsistent across versions and platforms. Explicitly tracking these files
    // ensures reliable rebuilds when assets change, preventing stale embedded content
    // in preview windows.
    //
    // Reference: https://github.com/rust-lang/cargo/issues/1510
    // "Cargo rebuilds with include_str but not include_bytes"
    println!("cargo:rerun-if-changed=src/assets/default.css");
    println!("cargo:rerun-if-changed=src/assets/templates/index.html");
    
    tauri_build::build()
}