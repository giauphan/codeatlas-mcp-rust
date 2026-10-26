use serde_json::{json, Value};
use std::path::PathBuf;

fn get_claude_config_path() -> anyhow::Result<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        if let Ok(appdata) = std::env::var("APPDATA") {
            if !appdata.trim().is_empty() {
                return Ok(PathBuf::from(appdata)
                    .join("Claude")
                    .join("claude_desktop_config.json"));
            }
        }
        if let Ok(userprofile) = std::env::var("USERPROFILE") {
            if !userprofile.trim().is_empty() {
                return Ok(PathBuf::from(userprofile)
                    .join("AppData")
                    .join("Roaming")
                    .join("Claude")
                    .join("claude_desktop_config.json"));
            }
        }
    }

    let home = std::env::var("HOME")
        .map_err(|_| anyhow::anyhow!("HOME environment variable is not set"))?;
    if home.trim().is_empty() {
        anyhow::bail!("HOME environment variable is empty");
    }

    #[cfg(target_os = "macos")]
    let config_path = PathBuf::from(&home)
        .join("Library")
        .join("Application Support")
        .join("Claude")
        .join("claude_desktop_config.json");

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let config_path = PathBuf::from(&home)
        .join(".config")
        .join("Claude")
        .join("claude_desktop_config.json");

    Ok(config_path)
}

pub fn run_setup_claude(force: bool) -> anyhow::Result<()> {
    println!(
        "Configuring Claude Desktop MCP integration (force: {})...",
        force
    );
    let claude_config = get_claude_config_path()?;

    if let Some(parent) = claude_config.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut root: Value = if claude_config.exists() {
        let content = std::fs::read_to_string(&claude_config)?;
        if content.trim().is_empty() {
            json!({})
        } else {
            serde_json::from_str(&content).map_err(|e| {
                anyhow::anyhow!(
                    "Failed to parse existing Claude config at {}: {}. Refusing to overwrite.",
                    claude_config.display(),
                    e
                )
            })?
        }
    } else {
        json!({})
    };

    if !root.is_object() {
        anyhow::bail!(
            "Claude config at {} is not a JSON object. Refusing to overwrite.",
            claude_config.display()
        );
    }

    let mcp_servers = root
        .as_object_mut()
        .unwrap()
        .entry("mcpServers")
        .or_insert_with(|| json!({}));

    if !mcp_servers.is_object() {
        *mcp_servers = json!({});
    }

    let servers_map = mcp_servers.as_object_mut().unwrap();
    if servers_map.contains_key("codeatlas") && !force {
        println!(
            "  ℹ 'codeatlas' is already registered in Claude config. Use --force to overwrite."
        );
        return Ok(());
    }

    let current_exe = std::env::current_exe()?.to_string_lossy().to_string();

    servers_map.insert(
        "codeatlas".to_string(),
        json!({
            "command": current_exe,
            "args": []
        }),
    );

    let updated_content = serde_json::to_string_pretty(&root)?;
    if claude_config.exists() {
        let backup = claude_config.with_extension("json.bak");
        std::fs::copy(&claude_config, &backup)?;
        println!("  ℹ Backed up existing config to {}", backup.display());
    }
    std::fs::write(&claude_config, updated_content)?;

    println!(
        "  ✓ Registered 'codeatlas' MCP server into {}",
        claude_config.display()
    );
    Ok(())
}
