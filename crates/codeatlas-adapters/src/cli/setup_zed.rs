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

fn strip_jsonc_and_trailing_commas(input: &str) -> String {
    let mut without_comments = String::with_capacity(input.len());
    let mut in_string = false;
    let mut in_line_comment = false;
    let mut in_block_comment = false;
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if in_line_comment {
            if c == '\n' {
                in_line_comment = false;
                without_comments.push(c);
            }
            continue;
        }

        if in_block_comment {
            if c == '*' && chars.peek() == Some(&'/') {
                chars.next();
                in_block_comment = false;
            }
            continue;
        }

        if in_string {
            without_comments.push(c);
            if c == '\\' {
                if let Some(escaped) = chars.next() {
                    without_comments.push(escaped);
                }
            } else if c == '"' {
                in_string = false;
            }
            continue;
        }

        if c == '"' {
            in_string = true;
            without_comments.push(c);
        } else if c == '/' && chars.peek() == Some(&'/') {
            chars.next();
            in_line_comment = true;
        } else if c == '/' && chars.peek() == Some(&'*') {
            chars.next();
            in_block_comment = true;
        } else {
            without_comments.push(c);
        }
    }

    // Pass 2: remove trailing commas outside strings
    let chars: Vec<char> = without_comments.chars().collect();
    let mut result = String::with_capacity(chars.len());
    let mut i = 0;
    let mut in_str = false;

    while i < chars.len() {
        let c = chars[i];
        if in_str {
            result.push(c);
            if c == '\\' && i + 1 < chars.len() {
                i += 1;
                result.push(chars[i]);
            } else if c == '"' {
                in_str = false;
            }
            i += 1;
            continue;
        }

        if c == '"' {
            in_str = true;
            result.push(c);
            i += 1;
            continue;
        }

        if c == ',' {
            let mut j = i + 1;
            while j < chars.len() && chars[j].is_whitespace() {
                j += 1;
            }
            if j < chars.len() && (chars[j] == '}' || chars[j] == ']') {
                i += 1;
                continue;
            }
        }

        result.push(c);
        i += 1;
    }

    result
}

pub fn run_setup_zed(force: bool) -> anyhow::Result<()> {
    println!("Configuring Zed MCP integration (force: {})...", force);
    let zed_config = get_zed_config_path()?;

    if let Some(parent) = zed_config.parent() {
        std::fs::create_dir_all(parent)?;
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
    std::fs::write(&zed_config, updated_content)?;

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
