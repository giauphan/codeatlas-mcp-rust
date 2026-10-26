#!/usr/bin/env node

import { copyFileSync, existsSync, mkdirSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const root = join(__dirname, "..");
const exeExt = process.platform === "win32" ? ".exe" : "";
const binName = `codeatlas${exeExt}`;

const src = join(root, "target", "release", binName);
const dstDir = join(root, "bin");
const dst = join(dstDir, binName);

if (!existsSync(src)) {
  console.error(`Source binary not found at ${src}`);
  process.exit(1);
}

mkdirSync(dstDir, { recursive: true });
copyFileSync(src, dst);
console.log(`Successfully copied native binary to ${dst}`);
