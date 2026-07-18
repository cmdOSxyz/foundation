// Visibility: public
// schemas/permission-request.ts
// A PermissionRequest is raised before a sensitive step runs. The runtime pauses,
// shows the user exactly what will happen, and waits for Approve or Deny.
// This enforces the repo rule: sensitive actions require explicit user approval,
// and human authority is never removed.

import type { Id, Timestamp } from "./intent";
import type { RiskLevel } from "./capability";

/** The user's decision on a permission request. */
export type PermissionDecision =
  | "pending"   // waiting for the user
  | "approved"  // user allowed it
  | "denied"    // user refused it
  | "expired";  // no response within the allowed time

/**
 * A single request for the user to authorize one sensitive step before it runs.
 */
export interface PermissionRequest {
  /** Stable unique id for this request. */
  id: Id;

  /** The plan this request belongs to. */
  planId: Id;

  /** The specific step that needs approval. */
  stepId: Id;

  /** The capability being invoked, e.g. "filesystem". */
  capability: string;

  /** The action being invoked, e.g. "delete". */
  action: string;

  /** How risky the action is; shown prominently to the user. */
  risk: RiskLevel;

  /**
   * A clear, human-readable description of exactly what will happen if approved,
   * e.g. "Permanently delete 12 files in C:\\Users\\admin\\old".
   */
  summary: string;

  /** The concrete parameters that will be used, shown so the user can inspect them. */
  details: Record<string, unknown>;

  /** The user's current decision. */
  decision: PermissionDecision;

  /** When the request was raised. */
  requestedAt: Timestamp;

  /** When the user responded. Undefined while decision is "pending". */
  respondedAt?: Timestamp;

  /** Optional reason the user gave when denying. */
  denialReason?: string;
}