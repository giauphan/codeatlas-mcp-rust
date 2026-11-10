use serde_json::{json, Value};
use std::path::PathBuf;

fn get_zed_config_path() -> anyhow::Result<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        if let Ok(appdata) = std::env::var("APPDATA") {
            if !appdata.trim().is_empty() {
                return Ok(PathBuf::from(appdata).join("zed").join("settings.json"));
            }
        }
    }

    let home = std::env::var("HOME")
        .map_err(|_| anyhow::anyhow!("HOME environment variable is not set"))?;
    if home.trim().is_empty() {
        anyhow::bail!("HOME environment variable is empty");
    }

    let config_path = PathBuf::from(&home)
        .join(".config")
        .join("zed")
        .join("settings.json");

    Ok(config_path)
}

use super::jsonc_utils::strip_jsonc_and_trailing_commas;

pub fn run_setup_zed(force: bool) -> anyhow::Result<()> {
    println!("Configuring Zed MCP integration (force: {})...", force);
    let zed_config = get_zed_config_path()?;

    if let Some(parent) = zed_config.parent() {
        if !parent.exists() {
            let mut builder = std::fs::DirBuilder::new();
            builder.recursive(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::DirBuilderExt;
                builder.mode(0o700);
            }
            builder.create(parent)?;
        }
    }

    let mut root: Value = if zed_config.exists() {
        let content = std::fs::read_to_string(&zed_config)?;
        if content.trim().is_empty() {
            json!({})
        } else {
            let stripped = strip_jsonc_and_trailing_commas(&content);
            serde_json::from_str(&stripped).map_err(|e| {
                anyhow::anyhow!(
                    "Failed to parse existing Zed settings at {}: {}. Refusing to overwrite.",
                    zed_config.display(),
                    e
                )
            })?
        }
    } else {
        json!({})
    };

    if !root.is_object() {
        anyhow::bail!(
            "Zed settings at {} are not a JSON object. Refusing to overwrite.",
            zed_config.display()
        );
    }

    let context_servers = root
        .as_object_mut()
        .unwrap()
        .entry("context_servers")
        .or_insert_with(|| json!({}));

    if !context_servers.is_object() {
        anyhow::bail!(
            "'context_servers' in {} is not a JSON object. Refusing to overwrite.",
            zed_config.display()
        );
    }

    let servers_map = context_servers.as_object_mut().unwrap();
    if servers_map.contains_key("codeatlas") && !force {
        println!(
            "  ℹ 'codeatlas' is already registered in Zed settings. Use --force to overwrite."
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
    if zed_config.exists() {
        let backup = zed_config.with_extension("json.bak");
        std::fs::copy(&zed_config, &backup)?;
        println!("  ℹ Backed up existing settings to {}", backup.display());
        println!("  ⚠ Note: Existing inline comments are omitted in the updated JSON settings.");
    }

    let mut open_opts = std::fs::OpenOptions::new();
    open_opts.write(true).create(true).truncate(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        open_opts.mode(0o600);
    }

    let mut file = open_opts.open(&zed_config)?;
    use std::io::Write;
    file.write_all(updated_content.as_bytes())?;

    println!(
        "  ✓ Registered 'codeatlas' MCP context server into {}",
        zed_config.display()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_jsonc_and_trailing_commas() {
        let input = r#"{
            // line comment
            /* block
               comment */
            "key": "value, with comma",
            "nested": {
                "arr": [1, 2, 3,],
            },
        }"#;

        let cleaned = strip_jsonc_and_trailing_commas(input);
        let parsed: serde_json::Value =
            serde_json::from_str(&cleaned).expect("must parse cleaned JSON");
        assert_eq!(parsed["key"], "value, with comma");
        assert_eq!(parsed["nested"]["arr"][2], 3);
    }
}
