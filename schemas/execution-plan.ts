// Visibility: public
// schemas/execution-plan.ts
// The ExecutionPlan is produced from an Intent during the Planning stage:
// Intent -> Understanding -> Command -> [Execution Plan] -> Permission -> Runtime -> Verification -> Result
// It is an ordered graph of steps the runtime executes. The AI proposes it; the kernel executes it.
// The AI never executes directly â€” it only produces this plan.

import type { Id, Timestamp } from "./intent.js";

/** Lifecycle status of a whole plan. */
export type PlanStatus =
  | "draft"
  | "awaiting_permission"
  | "approved"
  | "executing"
  | "completed"
  | "failed"
  | "rolled_back";

/** Lifecycle status of a single step. */
export type StepStatus =
  | "pending"
  | "running"
  | "succeeded"
  | "failed"
  | "skipped";

/**
 * One unit of work in a plan. A step invokes exactly one capability action.
 * Steps never mutate system state directly â€” they call a capability, which the
 * runtime executes under permission control.
 */
export interface PlanStep {
  id: Id;
  description: string;
  capability: string;
  action: string;
  parameters: Record<string, unknown>;
  dependsOn: Id[];
  requiresPermission: boolean;
  status: StepStatus;
  error?: string;
}

/**
 * The full plan derived from an Intent: an ordered set of steps plus metadata.
 */
export interface ExecutionPlan {
  id: Id;
  intentId: Id;
  createdAt: Timestamp;
  status: PlanStatus;
  steps: PlanStep[];
  summary: string;
}
