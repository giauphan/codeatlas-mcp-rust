/**
 * Identifies host-injected context and skill-loader instructions that are not
 * project evidence. These strings can appear in Claude transcript JSONL but
 * must never be persisted or replayed as CodeAtlas memory.
 */
export function isNonProjectInstructionArtifact(value: unknown): boolean {
  const content = String(value ?? "");
  return /===\s*(?:End\s+)?Untrusted CodeAtlas historical reference\s*===/i.test(content) ||
    (/\bBase directory for this skill:\s*/i.test(content) && /<EXTREMELY[-_ ]IMPORTANT>/i.test(content));
}
