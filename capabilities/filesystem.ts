// Visibility: private
// capabilities/filesystem.ts
// The first real capability. It performs actual file operations on disk.
// Each action is small, verifiable, and (for writes) reversible where possible.
// The runtime decides WHEN to run these; this file only defines HOW.

import { readdir, rename as fsRename, stat, readFile, mkdir, rm } from "node:fs/promises";
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

    case "read": {
      const path = String(p.path ?? "");
      if (!path) throw new Error("read requires a 'path' parameter");
      const content = await readFile(path, "utf-8");
      const preview = content.length > 500 ? content.slice(0, 500) + "…" : content;
      return { ok: true, message: "Read " + content.length + " chars from " + path, data: preview };
    }

    case "mkdir": {
      const path = String(p.path ?? "");
      if (!path) throw new Error("mkdir requires a 'path' parameter");
      await mkdir(path, { recursive: true });
      return { ok: true, message: "Created folder " + path, data: { path } };
    }

    case "move": {
      const from = String(p.from ?? "");
      const to = String(p.to ?? "");
      if (!from || !to) throw new Error("move requires 'from' and 'to' parameters");
      await stat(from); // ensure source exists
      await fsRename(from, to);
      return { ok: true, message: "Moved " + from + " to " + to, data: { from, to } };
    }

    case "delete": {
      const path = String(p.path ?? "");
      if (!path) throw new Error("delete requires a 'path' parameter");
      await stat(path); // ensure it exists before deleting
      await rm(path, { recursive: false });
      return { ok: true, message: "Deleted " + path, data: { path } };
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
// --- Rollback ---------------------------------------------------------------
// For reversible actions, return a function that undoes the step. The runtime
// collects these and runs them in reverse order if a later step fails.
// Returns null for actions that need no undo (e.g. read-only).

export type UndoFn = () => Promise<void>;

/** Produce an undo function for a step that already succeeded, or null. */
export function undoForFilesystemStep(step: PlanStep): UndoFn | null {
  const p = step.parameters;

  switch (step.action) {
    case "rename": {
      const from = String(p.from ?? "");
      const to = String(p.to ?? "");
      const target =
        to.includes("/") || to.includes("\\") ? to : join(dirname(from), to);
      // Undo = rename back: target -> from.
      return async () => {
        await fsRename(target, from);
      };
    }

    case "list":
      return null; // read-only, nothing to undo

    default:
      return null;
  }
}

// --- Inspection (context gathering) -----------------------------------------
// Returns real facts about a path so the agent can describe it accurately
// instead of guessing. Read-only: never modifies anything.

import { stat as statPath } from "node:fs/promises";
import { resolve } from "node:path";

export interface PathInfo {
  exists: boolean;
  fullPath: string;
  type?: "file" | "folder";
  sizeBytes?: number;
  modified?: string;
  content?: string;
}

export async function inspectPath(p: string): Promise<PathInfo> {
  const fullPath = resolve(p);
  try {
    const s = await statPath(fullPath);
    const info: PathInfo = {
      exists: true,
      fullPath,
      type: s.isDirectory() ? "folder" : "file",
      sizeBytes: s.size,
      modified: s.mtime.toISOString(),
    };

    // If it's a small text-ish file, include its content so the agent can read it.
    if (!s.isDirectory() && s.size <= 20000) {
      const textExt = /\.(txt|md|json|csv|log|js|ts|html|css|xml|yml|yaml|ini)$/i;
      if (textExt.test(fullPath)) {
        try {
          info.content = await readFile(fullPath, "utf-8");
        } catch {
          // binary or unreadable; skip content
        }
      }
    }
    return info;
  } catch {
    return { exists: false, fullPath };
  }
}