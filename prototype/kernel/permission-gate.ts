// Visibility: private
// kernel/permission-gate.ts
// Before a sensitive step runs, the gate asks the user to Approve or Deny.
// The decision function is injected, so the same gate works with a console prompt now
// and a real UI dialog later. This enforces: sensitive actions require explicit approval.

import type { PlanStep, PermissionRequest } from "../../schemas/index.js";

/** A function that shows a request to the user and returns their decision. */
export type Approver = (request: PermissionRequest) => Promise<boolean>;

/** Generate a short random id. */
function id(prefix: string): string {
  return prefix + "-" + Math.random().toString(36).slice(2, 8);
}

/**
 * Decides whether a step may run. Steps that do not require permission pass
 * automatically. Steps that do are turned into a PermissionRequest and sent to
 * the injected approver.
 */
export class PermissionGate {
  constructor(private readonly approve: Approver) {}

  /** Returns true if the step is allowed to run. */
  async check(planId: string, step: PlanStep): Promise<boolean> {
    if (!step.requiresPermission) return true;

    const request: PermissionRequest = {
      id: id("perm"),
      planId,
      stepId: step.id,
      capability: step.capability,
      action: step.action,
      risk: "reversible",
      summary:
        "cmdOS wants to run " + step.capability + "." + step.action + " — " + step.description,
      details: step.parameters,
      decision: "pending",
      requestedAt: new Date().toISOString(),
    };

    return this.approve(request);
  }
}