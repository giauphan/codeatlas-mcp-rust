#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import { homedir } from "node:os";
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

// Locate cargo in fixed, standard system directories to prevent untrusted PATH search
function findCargo() {
  const cargoName = `cargo${exeExt}`;
  const candidates = [
    join(homedir(), ".cargo", "bin", cargoName),
    join("/usr/local/cargo/bin", cargoName),
    join("/usr/local/bin", cargoName),
    join("/usr/bin", cargoName),
    join("/bin", cargoName),
  ];
  for (const candidate of candidates) {
    if (existsSync(candidate)) {
      return candidate;
    }
  }
  return null;
}

const cargoBin = findCargo();
const safePath = [
  join(homedir(), ".cargo", "bin"),
  "/usr/local/cargo/bin",
  "/usr/local/bin",
  "/usr/bin",
  "/bin",
].join(process.platform === "win32" ? ";" : ":");

const safeEnv = {
  ...process.env,
  PATH: safePath,
};

// Attempt to compile with Cargo if available
if (cargoBin) {
  try {
    const cargoCheck = spawnSync(cargoBin, ["--version"], {
      stdio: "ignore",
      env: safeEnv,
    });
    if (cargoCheck.status === 0) {
      console.log("Compiling CodeAtlas native Rust engine via Cargo for current platform...");
      const build = spawnSync(cargoBin, ["build", "--release", "--locked"], {
        cwd: root,
        stdio: "inherit",
        env: safeEnv,
      });
      if (build.status === 0) {
        const copy = spawnSync(process.execPath, [join(__dirname, "copy-binary.js")], {
          cwd: root,
          stdio: "inherit",
          env: safeEnv,
        });
        if (copy.status !== 0) {
          console.warn("CodeAtlas: failed to copy the compiled binary into bin/.");
        }
      } else {
        console.warn("CodeAtlas: cargo build failed. The native engine is not available.");
      }
    } else {
      console.log("Cargo not functional. CodeAtlas native engine can be built with Cargo or downloaded.");
    }
  } catch {
    // Silent fallback: launcher will prompt when executed if binary is missing
  }
} else {
  console.log("Cargo not detected. CodeAtlas native engine can be built with Cargo or downloaded.");
}
