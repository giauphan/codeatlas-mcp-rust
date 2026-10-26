#!/usr/bin/env node

/**
 * CodeAtlas Native Engine Launcher
 * Thin, high-performance launcher that connects npm / npx invocations
 * directly to the native Rust engine.
 */

import { basename } from "node:path";
import { runLauncher } from "../index.js";

const invokedAs = basename(process.argv[1] ?? "").replace(/\.js$/, "");
const extraArgs = invokedAs === "codeatlas-setup-claude" ? ["setup-claude"] : [];
runLauncher(extraArgs);
