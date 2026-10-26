import { spawnSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const exeExt = process.platform === "win32" ? ".exe" : "";
const binName = `codeatlas${exeExt}`;

export function getBinaryPath() {
  const candidates = [
    join(__dirname, "bin", binName),
    join(__dirname, "target", "release", binName),
    join(__dirname, "target", "debug", binName),
  ];
  return candidates.find((p) => existsSync(p)) ?? null;
}

let pkgVersion = "0.3.1";
try {
  const pkg = JSON.parse(readFileSync(join(__dirname, "package.json"), "utf8"));
  if (pkg.version) {
    pkgVersion = pkg.version;
  }
} catch {
  // Use fallback
}

export const version = pkgVersion;

export function runLauncher(extraArgs = []) {
  if (process.argv.includes("--version") || process.argv.includes("-v")) {
    console.log(pkgVersion);
    process.exit(0);
  }

  const binaryPath = getBinaryPath();
  if (!binaryPath) {
    console.error(
      "Error: CodeAtlas native binary not found. Please build with 'cargo build --release' or check your installation."
    );
    process.exit(1);
  }

  const forwardedArgs = [...extraArgs, ...process.argv.slice(2)];
  const child = spawnSync(binaryPath, forwardedArgs, {
    stdio: "inherit",
    env: process.env,
  });

  if (child.error) {
    console.error(`Error: Failed to execute CodeAtlas native binary at ${binaryPath}: ${child.error.message}`);
    process.exit(1);
  }

  if (child.signal) {
    console.error(`CodeAtlas native binary terminated by signal ${child.signal}`);
    process.exit(1);
  }

  process.exit(child.status ?? 1);
}
