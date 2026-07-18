// Visibility: private
// capabilities/filesystem.ts
// The first real capability. It performs actual file operations on disk.
// Each action is small, verifiable, and (for writes) reversible where possible.
// The runtime decides WHEN to run these; this file only defines HOW.

import { readdir, rename as fsRename, stat } from "node:fs/promises";
import { join, dirname, basename } from "node:path";
import type { PlanStep } from "../schemas/index.js";

/** Result of running one filesystem action. */
export interface FsResult {
  ok: boolean;
  message: string;
  data?: unknown;
}

/**
 * Runs one filesystem step. Throws on error so the Executor records a failure.
 * Supported actions: "list", "rename".
 */
export async function runFilesystemStep(step: PlanStep): Promise<FsResult> {
  const p = step.parameters;

  switch (step.action) {
    case "list": {
      const path = String(p.path ?? "");
      if (!path) throw new Error("list requires a 'path' parameter");
      const entries = await readdir(path);
      return {
        ok: true,
        message: "Listed " + entries.length + " entries in " + path,
        data: entries,
      };
    }

    case "rename": {
      const from = String(p.from ?? "");
      const to = String(p.to ?? "");
      if (!from || !to) throw new Error("rename requires 'from' and 'to' parameters");

      // Safety: make sure the source exists before touching anything.
      await stat(from); // throws if 'from' does not exist

      // Rename within the same folder unless 'to' is an absolute path.
      const target = to.includes("/") || to.includes("\\") ? to : join(dirname(from), to);

      await fsRename(from, target);
      return {
        ok: true,
        message: "Renamed " + basename(from) + " to " + basename(target),
        data: { from, to: target },
      };
    }

    default:
      throw new Error("filesystem: unknown action '" + step.action + "'");
  }
}
// --- Verification -----------------------------------------------------------
// After a step runs, confirm the real filesystem state matches what was intended.
// Returns { ok, message }. ok=false means the action did not achieve its goal.

import { access } from "node:fs/promises";

/** True if a path exists, false otherwise. */
async function exists(path: string): Promise<boolean> {
  try {
    await access(path);
    return true;
  } catch {
    return false;
  }
}

/** Verify that a completed step actually did what it claimed. */
export async function verifyFilesystemStep(step: PlanStep): Promise<FsResult> {
  const p = step.parameters;

  switch (step.action) {
    case "rename": {
      const from = String(p.from ?? "");
      const to = String(p.to ?? "");
      const target =
        to.includes("/") || to.includes("\\") ? to : join(dirname(from), to);

      const newExists = await exists(target);
      const oldGone = !(await exists(from));

      if (newExists && oldGone) {
        return { ok: true, message: "Verified: rename succeeded" };
      }
      return {
        ok: false,
        message:
          "Verification FAILED: newExists=" + newExists + " oldGone=" + oldGone,
      };
    }

    case "list":
      // Read-only: nothing to verify.
      return { ok: true, message: "Verified: read-only action" };

    default:
      return { ok: true, message: "No verification for action " + step.action };
  }
}