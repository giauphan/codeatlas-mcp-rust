#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const root = join(__dirname, "..");
const exeExt = process.platform === "win32" ? ".exe" : "";
const binName = `codeatlas${exeExt}`;
const binPath = join(__dirname, binName);

// Check if bundled binary exists AND is compatible with the current host
if (existsSync(binPath)) {
  try {
    const testRun = spawnSync(binPath, ["--version"], { stdio: "ignore" });
    if (testRun.status === 0 && !testRun.error) {
      process.exit(0);
    }
  } catch {
    // Incompatible binary, proceed to compile
  }
}

// Attempt to compile with Cargo if available
try {
  const cargoCheck = spawnSync("cargo", ["--version"], { stdio: "ignore" });
  if (cargoCheck.status === 0) {
    console.log("Compiling CodeAtlas native Rust engine via Cargo for current platform...");
    const build = spawnSync("cargo", ["build", "--release"], {
      cwd: root,
      stdio: "inherit",
    });
    if (build.status === 0) {
      const copy = spawnSync(process.execPath, [join(__dirname, "copy-binary.js")], {
        cwd: root,
        stdio: "inherit",
      });
      if (copy.status !== 0) {
        console.warn("CodeAtlas: failed to copy the compiled binary into bin/.");
      }
    } else {
      console.warn("CodeAtlas: cargo build failed. The native engine is not available.");
    }
  } else {
    console.log("Cargo not detected. CodeAtlas native engine can be built with Cargo or downloaded.");
  }
} catch {
  // Silent fallback: launcher will prompt when executed if binary is missing
}
