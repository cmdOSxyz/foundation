// Visibility: public
// schemas/planner.ts
// The Planner contract: turns a validated Intent into an ExecutionPlan.
// Implementations may be a mock, a rule engine, or an AI model. The rest of the system
// depends only on this contract, never on a specific provider (provider-neutral).

import type { Intent } from "./intent.js";
import type { ExecutionPlan } from "./execution-plan.js";

/** Anything that can produce an ExecutionPlan from an Intent. */
export interface Planner {
  /** Produce a plan for the given intent. May call an AI model, a rule engine, etc. */
  plan(intent: Intent): Promise<ExecutionPlan>;
}