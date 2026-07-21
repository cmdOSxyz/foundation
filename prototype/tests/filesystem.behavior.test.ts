// Visibility: private
// prototype/tests/filesystem.behavior.test.ts
// BEHAVIOR CONTRACT for the filesystem capability.
// This suite defines the observable behavior every future implementation
// (including the Rust cmd-transaction + cap-files port) MUST reproduce.
// Run: npm test

import { test, before, after } from "node:test";
import assert from "node:assert/strict";
import { mkdtemp, writeFile, rm, readdir, access } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import {
  runFilesystemStep,
  verifyFilesystemStep,
  undoForFilesystemStep,
  dryRunFilesystemStep,
  inspectPath,
} from "../capabilities/filesystem.js";
import type { PlanStep } from "../../schemas/index.js";

let dir: string;

function step(action: string, parameters: Record<string, unknown>): PlanStep {
  return {
    id: "step-test",
    description: "behavior contract test step",
    capability: "filesystem",
    action,
    parameters,
    dependsOn: [],
    requiresPermission: false,
    status: "pending",
  } as PlanStep;
}

async function exists(p: string): Promise<boolean> {
  try { await access(p); return true; } catch { return false; }
}

before(async () => {
  dir = await mkdtemp(join(tmpdir(), "cmdos-behavior-"));
});

after(async () => {
  await rm(dir, { recursive: true, force: true });
});

// --- CONTRACT 1: dry-run is read-only and truthful --------------------------

test("dry-run of rename describes the change and touches nothing", async () => {
  const src = join(dir, "a.txt");
  await writeFile(src, "hello");
  const s = step("rename", { from: src, to: "b.txt" });

  const preview = await dryRunFilesystemStep(s);

  assert.ok(preview.summary.includes(src), "summary names the source");
  assert.equal(preview.reversible, true, "rename is declared reversible");
  assert.equal(await exists(src), true, "dry-run must not modify the filesystem");
  assert.equal(await exists(join(dir, "b.txt")), false, "target not created by preview");
});

test("dry-run warns when the source is missing", async () => {
  const s = step("rename", { from: join(dir, "ghost.txt"), to: "x.txt" });
  const preview = await dryRunFilesystemStep(s);
  assert.ok(preview.warnings.some((w) => w.includes("does not exist")));
});

// --- CONTRACT 2: execute -> verify -> undo round-trip ------------------------

test("rename: execute succeeds, verify confirms, undo restores exactly", async () => {
  const src = join(dir, "report.txt");
  await writeFile(src, "q1 numbers");
  const s = step("rename", { from: src, to: "report-final.txt" });
  const target = join(dir, "report-final.txt");

  const result = await runFilesystemStep(s);
  assert.equal(result.ok, true);
  assert.equal(await exists(target), true, "target exists after execute");
  assert.equal(await exists(src), false, "source gone after execute");

  const verified = await verifyFilesystemStep(s);
  assert.equal(verified.ok, true, "verify confirms the intended end state");

  const undo = undoForFilesystemStep(s);
  assert.ok(undo, "a reversible action must provide an undo function");
  await undo!();
  assert.equal(await exists(src), true, "undo restores the original path");
  assert.equal(await exists(target), false, "undo removes the new path");
});

test("verify reports failure when the world does not match the claim", async () => {
  // Claim a rename that never happened.
  const s = step("rename", {
    from: join(dir, "never-a.txt"),
    to: "never-b.txt",
  });
  const verified = await verifyFilesystemStep(s);
  assert.equal(verified.ok, false, "verify must not rubber-stamp");
});

// --- CONTRACT 3: read-only actions need no undo ------------------------------

test("list executes, verifies as read-only, and returns null undo", async () => {
  await writeFile(join(dir, "one.txt"), "1");
  const s = step("list", { path: dir });

  const result = await runFilesystemStep(s);
  assert.equal(result.ok, true);
  assert.ok(Array.isArray(result.data));

  const verified = await verifyFilesystemStep(s);
  assert.equal(verified.ok, true);

  assert.equal(undoForFilesystemStep(s), null, "read-only actions have no undo");
});

// --- CONTRACT 4: delete is recoverable (trash), never destructive ------------

test("delete moves to trash instead of destroying, and reports recoverability", async () => {
  const victim = join(dir, "old-notes.txt");
  await writeFile(victim, "keep me safe");
  const s = step("delete", { path: victim });

  const preview = await dryRunFilesystemStep(s);
  assert.equal(preview.reversible, true, "delete is declared recoverable");

  const result = await runFilesystemStep(s);
  assert.equal(result.ok, true);
  assert.ok(result.message.toLowerCase().includes("recoverable"));
  assert.equal(await exists(victim), false, "original path is gone");

  const data = result.data as { trashedPath: string };
  assert.equal(await exists(data.trashedPath), true, "content survives in trash");
});

// --- CONTRACT 5: failures throw; nothing half-happens -------------------------

test("rename of a missing source throws and creates nothing", async () => {
  const s = step("rename", { from: join(dir, "nope.txt"), to: "yes.txt" });
  await assert.rejects(() => runFilesystemStep(s));
  assert.equal(await exists(join(dir, "yes.txt")), false, "no partial effect on failure");
});

// --- CONTRACT 6: inspection tells the truth ----------------------------------

test("inspectPath reports existence, type, and small text content", async () => {
  const f = join(dir, "info.md");
  await writeFile(f, "# hello");
  const info = await inspectPath(f);
  assert.equal(info.exists, true);
  assert.equal(info.type, "file");
  assert.equal(info.content, "# hello");

  const missing = await inspectPath(join(dir, "void.md"));
  assert.equal(missing.exists, false);
});
