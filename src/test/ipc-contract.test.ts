import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { join } from "node:path";

// Guards the IPC boundary: every command the UI invokes must be registered in the Tauri
// handler list, otherwise the call fails at runtime with "command not found".
const root = join(__dirname, "..", "..");
const apiSource = readFileSync(join(root, "src", "lib", "api.ts"), "utf8");
const libSource = readFileSync(join(root, "src-tauri", "src", "lib.rs"), "utf8");

function invokedCommands(): string[] {
  const names = new Set<string>();
  for (const match of apiSource.matchAll(/call<[^>]*>\(\s*"([a-z_]+)"/g)) {
    names.add(match[1]);
  }
  return [...names].sort();
}

function registeredCommands(): string[] {
  const block = libSource.match(/generate_handler!\[([\s\S]*?)\]/);
  if (!block) throw new Error("generate_handler! block not found in lib.rs");
  return [...block[1].matchAll(/commands::([a-z_]+)/g)].map((m) => m[1]).sort();
}

describe("IPC contract", () => {
  it("finds the invoked commands in the API bridge", () => {
    expect(invokedCommands().length).toBeGreaterThan(20);
  });

  it("every command invoked from the UI is registered in the Tauri backend", () => {
    const registered = new Set(registeredCommands());
    const missing = invokedCommands().filter((name) => !registered.has(name));
    expect(missing).toEqual([]);
  });

  it("registered commands are all reachable from the UI", () => {
    const invoked = new Set(invokedCommands());
    const unused = registeredCommands().filter((name) => !invoked.has(name));
    expect(unused).toEqual([]);
  });
});
