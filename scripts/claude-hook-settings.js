/** One compatibility-preserving Claude hook installer for CLI, npm and legacy bridges. */
import { existsSync, lstatSync, mkdirSync, readFileSync, writeFileSync, copyFileSync, chmodSync, renameSync } from 'node:fs';
import { homedir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const packageRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const routes = [
  ['PreToolUse', 'task-router', '*'],
  ['SessionStart', 'brain-context'],
  ['UserPromptSubmit', 'brain-context'],
  ['PostToolUse', 'brain-save', '*'],
  ['PostToolUseFailure', 'brain-save', '*'],
  ['Stop', 'brain-save'],
  ['PreCompact', 'brain-save'],
];

const quote = (value) => "'" + value.replaceAll("'", String.raw`'\''`) + "'";
const cliPath = existsSync(join(packageRoot, 'index.js'))
  ? join(packageRoot, 'index.js')
  : join(packageRoot, 'dist', 'index.js');
const nativeCommand = (route) => `${quote(process.execPath)} ${quote(cliPath)} hook ${route}`;

function assertSafeDestination(file, label) {
  try {
    const entry = lstatSync(file); // NOSONAR S8707: this is a pre-write symlink/type guard; callers reject unsafe destinations.
    if (entry.isSymbolicLink() || !entry.isFile()) {
      throw new Error(`${label} must be a regular file`);
    }
  } catch (error) {
    if (error?.code !== 'ENOENT') throw error;
  }
}

function assertSafeDirectory(directory, label) {
  try {
    const entry = lstatSync(directory); // NOSONAR S8707: this is a pre-write symlink/type guard; callers reject unsafe directories.
    if (entry.isSymbolicLink() || !entry.isDirectory()) {
      throw new Error(`${label} must be a real directory`);
    }
  } catch (error) {
    if (error?.code !== 'ENOENT') throw error;
  }
}

function extractHookCommand(hook) {
  if (typeof hook === 'string') {
    return hook;
  }
  if (typeof hook?.command === 'string') {
    return hook.command.trim();
  }
  return '';
}

function ownsRoute(hook, route) {
  const command = extractHookCommand(hook);
  const args = Array.isArray(hook?.args) ? hook.args : [];
  if (command === nativeCommand(route)) return true;

  // Native routes written by earlier installs embed different node/CLI paths.
  if (new RegExp(String.raw`[\\/]index\.js["']?\s+hook\s+${route}\s*$`).test(command)) {
    return true;
  }

  const executable = command.replaceAll('\\', '/').split('/').at(-1);
  if (/^codeatlas(?:-enterprise|-mcp)?$/.test(executable) && args.includes(route)) {
    return true;
  }

  if (new RegExp(String.raw`(?:^|[\s/])codeatlas(?:-enterprise|-mcp)?\s+(?:hook\s+)?${route}$`).test(command)) {
    return true;
  }

  // Remove only known CodeAtlas compatibility assets, never arbitrary vendor shell hooks.
  const normalizedCommand = command.replaceAll('\\', '/');
  if (new RegExp(String.raw`(?:^|/)\.claude/hooks/${route}\.sh(?:["']|$)`).test(normalizedCommand)) {
    return true;
  }

  return false;
}

function formatLegacyGroup(group) {
  if (typeof group === 'string') {
    return { hooks: [{ type: 'command', command: group }] };
  }
  if (group && typeof group === 'object' && typeof group.command === 'string') {
    return { hooks: [group] };
  }
  return group;
}

function processHookGroup(group, route, preserved) {
  if (group && typeof group === 'object' && Array.isArray(group.hooks)) {
    const hooks = group.hooks.filter((hook) => !ownsRoute(hook, route));
    if (hooks.length === group.hooks.length) {
      preserved.push(group);
    } else if (hooks.length > 0) {
      preserved.push({ ...group, hooks });
    }
  } else if (!ownsRoute(group, route)) {
    preserved.push(formatLegacyGroup(group));
  }
}

export function mergeSettings(existing = {}) {
  if (!existing || typeof existing !== 'object' || Array.isArray(existing)) {
    throw new Error('Claude settings root must be an object');
  }

  const merged = structuredClone(existing);
  if (merged.hooks !== undefined && (!merged.hooks || typeof merged.hooks !== 'object' || Array.isArray(merged.hooks))) {
    throw new Error('settings.hooks must be an object');
  }

  merged.hooks ??= {};

  for (const [event, route, matcher] of routes) {
    if (merged.hooks[event] !== undefined && !Array.isArray(merged.hooks[event])) {
      throw new Error(`settings.hooks.${event} must be an array`);
    }

    const preserved = [];
    for (const group of merged.hooks[event] ?? []) {
      processHookGroup(group, route, preserved);
    }

    const commandEntry = { type: 'command', command: nativeCommand(route) };
    const matcherObj = matcher ? { matcher } : {};
    merged.hooks[event] = [...preserved, { ...matcherObj, hooks: [commandEntry] }];
  }

  return merged;
}

export function installClaudeHooks(settingsFile = join(homedir(), '.claude', 'settings.json')) {
  const settingsPath = resolve(settingsFile);
  const segments = settingsPath.split(/[\\/]/);
  if (segments.at(-1) !== 'settings.json' || segments.at(-2) !== '.claude') {
    throw new Error('Claude settings path must end in .claude/settings.json');
  }

  assertSafeDestination(settingsPath, 'Claude settings');
  const settings = existsSync(settingsPath) ? JSON.parse(readFileSync(settingsPath, 'utf8')) : {}; // NOSONAR S8707
  const merged = mergeSettings(settings);

  const hooksDir = join(dirname(settingsPath), 'hooks');
  assertSafeDirectory(dirname(settingsPath), 'Claude settings directory');
  assertSafeDirectory(hooksDir, 'Claude hooks directory');

  const sourceHooks = join(packageRoot, 'src', 'cli', 'hooks');
  const assets = ['brain-save.sh', 'brain-context.sh', 'task-router.sh'];
  for (const file of assets) {
    if (!existsSync(join(sourceHooks, file))) {
      throw new Error(`Missing hook asset: ${file}`);
    }
  }

  mkdirSync(hooksDir, { recursive: true, mode: 0o700 }); // NOSONAR S8707
  assertSafeDirectory(dirname(settingsPath), 'Claude settings directory');
  assertSafeDirectory(hooksDir, 'Claude hooks directory');

  for (const file of assets) {
    const target = join(hooksDir, file);
    assertSafeDestination(target, 'Claude hook asset');
    copyFileSync(join(sourceHooks, file), target); // NOSONAR S8707
    chmodSync(target, 0o700); // NOSONAR S8707
  }

  const hookRoutes = ['brain-context', 'brain-save', 'task-router'];
  const caseStatements = hookRoutes.map((route) => {
    const scriptPath = join(hooksDir, route + '.sh');
    return '  ' + route + ') exec ' + quote(scriptPath) + ' ;;';
  });

  const wrapper = [
    '#!/bin/bash',
    'set -euo pipefail',
    `export CODEATLAS_CLI_PATH=${quote(cliPath)}`,
    'command_name="${1:-}"',
    'if [ "$command_name" = "hook" ]; then shift; command_name="${1:-}"; fi',
    'case "$command_name" in',
    ...caseStatements,
    '  *) echo "usage: codeatlas hook <brain-context|brain-save|task-router>" >&2; exit 2 ;;',
    'esac',
    '',
  ].join('\n');

  const wrapperPath = join(hooksDir, 'codeatlas');
  assertSafeDestination(wrapperPath, 'Claude hook wrapper');
  writeFileSync(wrapperPath, wrapper, { mode: 0o700 }); // NOSONAR S8707
  chmodSync(wrapperPath, 0o700); // NOSONAR S8707

  const temporary = `${settingsPath}.${process.pid}.tmp`;
  assertSafeDestination(temporary, 'Claude settings temporary file');
  writeFileSync(temporary, `${JSON.stringify(merged, null, 2)}\n`, { mode: 0o600 }); // NOSONAR S8707
  renameSync(temporary, settingsPath); // NOSONAR S8707
}
