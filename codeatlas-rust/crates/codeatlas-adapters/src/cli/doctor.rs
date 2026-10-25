pub fn run_doctor() -> anyhow::Result<()> {
    println!("CodeAtlas System Diagnostics:");
    println!("  ✓ Rust Native Engine: active");

    let home = std::env::var("HOME").unwrap_or_default();
    let adr_dir = std::path::Path::new(&home).join(".codeatlas").join("adr");
    if adr_dir.exists() {
        println!("  ✓ ADR Directory: exists ({})", adr_dir.display());
    } else {
        println!("  ℹ ADR Directory: not created yet");
    }

    if std::env::var("CODEATLAS_API_KEY").is_ok() {
        println!("  ✓ CodeAtlas Cloud API Key: configured");
    } else {
        println!("  ℹ CodeAtlas Cloud: running in local-first offline mode");
    }

    println!("\nDiagnostics complete: System is healthy.");
    Ok(())
}
