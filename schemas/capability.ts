// Visibility: public
// schemas/capability.ts
// A Capability is the core execution primitive: a versioned contract plus (elsewhere)
// an implementation. Agents invoke Capabilities â€” never Plugins â€” and only the runtime
// executes them, under permission control. This file defines the CONTRACT shape only;
// concrete implementations (e.g. filesystem) live under capabilities/ in the private core.

import type { Id } from "./intent.js";

/** How risky an action is, used to decide whether it needs user approval. */
export type RiskLevel =
  | "read_only"   // observes state, changes nothing (e.g. list files)
  | "reversible"  // changes state but can be undone (e.g. rename with backup)
  | "destructive" // hard or impossible to undo (e.g. permanent delete)
  | "external";   // affects the outside world (e.g. send email, network call)

/**
 * Describes a single action a capability can perform. This is the contract the
 * AI planner reads to know what actions exist and what parameters they take.
 */
export interface CapabilityAction {
  /** Action name, unique within the capability, e.g. "rename". */
  name: string;

  /** Human-readable description of what the action does. */
  description: string;

  /** Names of parameters this action expects. */
  parameters: string[];

  /** How risky the action is; drives the permission requirement. */
  risk: RiskLevel;
}

/**
 * The contract for a capability. A capability groups related actions under one
 * name (e.g. "filesystem" with actions list/read/rename/move).
 */
export interface Capability {
  /** Stable unique id for this capability. */
  id: Id;

  /** Capability name used by plan steps, e.g. "filesystem". */
  name: string;

  /** Semantic version of this capability's contract, e.g. "1.0.0". */
  version: string;

  /** Human-readable description of the capability's domain. */
  description: string;

  /** The actions this capability exposes. */
  actions: CapabilityAction[];
}
