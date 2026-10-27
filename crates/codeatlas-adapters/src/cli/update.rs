//! CLI Update Command: check for updates and upgrade CodeAtlas from crates.io

use anyhow::{bail, Context, Result};
use std::process::Command;

#[derive(serde::Deserialize, Debug)]
struct CratesResponse {
    #[serde(rename = "crate")]
    krate: CrateInfo,
}

#[derive(serde::Deserialize, Debug)]
struct CrateInfo {
    max_version: String,
}

/// Checks crates.io for the latest version and optionally upgrades via `cargo install`.
pub async fn run_update(check_only: bool, force: bool) -> Result<()> {
    let current_version = env!("CARGO_PKG_VERSION");
    println!("Checking for CodeAtlas updates...");
    println!("  Installed version : v{}", current_version);

    let latest_version = match fetch_latest_from_crates_io(current_version).await {
        Ok(v) => v,
        Err(e) => match fetch_latest_via_curl() {
            Ok(v) => v,
            Err(_) => return Err(e),
        },
    };

    println!("  Latest on crates.io: v{}", latest_version);

    let is_newer = is_newer_version(&latest_version, current_version);

    if check_only {
        if is_newer {
            println!(
                "\n⚡ Update available: v{} -> v{}",
                current_version, latest_version
            );
            println!("Run `codeatlas update` to install the latest release.");
        } else {
            println!(
                "\n✓ CodeAtlas is already at the latest version (v{}).",
                current_version
            );
        }
        return Ok(());
    }

    if !is_newer && !force {
        println!(
            "\n✓ CodeAtlas is already at the latest version (v{}).",
            current_version
        );
        println!("Use `codeatlas update --force` to reinstall the current version.");
        return Ok(());
    }

    println!(
        "\n⚡ Upgrading CodeAtlas to v{} via cargo install...",
        latest_version
    );

    let status = Command::new("cargo")
        .args(["install", "codeatlas-cli", "--force"])
        .status()
        .context("Failed to execute `cargo install codeatlas-cli --force`")?;

    if status.success() {
        println!("\n✓ CodeAtlas successfully updated to v{}!", latest_version);
    } else {
        bail!(
            "`cargo install codeatlas-cli --force` failed with exit status: {:?}",
            status.code()
        );
    }

    Ok(())
}

async fn fetch_latest_from_crates_io(current_version: &str) -> Result<String> {
    let client = reqwest::Client::builder()
        .user_agent(format!("codeatlas-cli/{}", current_version))
        .build()?;

    let resp = client
        .get("https://crates.io/api/v1/crates/codeatlas-cli")
        .send()
        .await
        .context("Failed to connect to crates.io registry")?;

    if !resp.status().is_success() {
        bail!("Failed to query crates.io registry: HTTP {}", resp.status());
    }

    let data: CratesResponse = resp
        .json()
        .await
        .context("Failed to parse crates.io registry response")?;
    Ok(data.krate.max_version)
}

fn fetch_latest_via_curl() -> Result<String> {
    let output = Command::new("curl")
        .args([
            "-s",
            "-H",
            "User-Agent: codeatlas-cli",
            "https://crates.io/api/v1/crates/codeatlas-cli",
        ])
        .output()
        .context("Failed to run curl")?;

    if !output.status.success() {
        bail!("curl failed to fetch crates.io data");
    }

    let data: CratesResponse =
        serde_json::from_slice(&output.stdout).context("Failed to parse curl JSON response")?;
    Ok(data.krate.max_version)
}

/// Helper function to compare semantic versions.
pub fn is_newer_version(latest: &str, current: &str) -> bool {
    let parse = |v: &str| -> Option<(u64, u64, u64)> {
        let v_trimmed = v.trim().trim_start_matches('v');
        let mut parts = v_trimmed.split('.');
        let major = parts.next()?.parse().ok()?;
        let minor = parts.next()?.parse().ok()?;
        let patch = parts.next()?.parse().ok()?;
        Some((major, minor, patch))
    };

    match (parse(latest), parse(current)) {
        (Some(l), Some(c)) => l > c,
        _ => latest.trim() != current.trim(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_newer_version() {
        assert!(is_newer_version("0.3.2", "0.3.1"));
        assert!(is_newer_version("0.4.0", "0.3.1"));
        assert!(is_newer_version("1.0.0", "0.3.1"));
        assert!(is_newer_version("v0.3.2", "v0.3.1"));

        assert!(!is_newer_version("0.3.1", "0.3.1"));
        assert!(!is_newer_version("v0.3.1", "v0.3.1"));
        assert!(!is_newer_version("0.3.0", "0.3.1"));
        assert!(!is_newer_version("0.2.9", "0.3.1"));
    }
}
