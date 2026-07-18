// Visibility: public
// schemas/intent.ts
// The Intent is the first artifact in the canonical loop:
// Intent -> Understanding -> Command -> Execution Plan -> Permission -> Runtime -> Verification -> Result
// It is the structured, validated representation of what the user wants.

/** A unique identifier (UUID v4 recommended). */
export type Id = string;

/** ISO-8601 timestamp string, e.g. "2026-07-18T15:30:00Z". */
export type Timestamp = string;

/** Where the intent came from. */
export type IntentSource = "user_command" | "voice" | "scheduled" | "agent";

/** Lifecycle status of an intent as it moves through the loop. */
export type IntentStatus =
  | "received"    // captured from the user, not yet understood
  | "understood"  // parsed and validated into a clear objective
  | "planned"     // an ExecutionPlan has been produced
  | "rejected";   // could not be understood or is not permitted

/**
 * A single expressed goal from the user, in their own words,
 * plus the structured understanding cmdOS derives from it.
 */
export interface Intent {
  /** Stable unique id for this intent. */
  id: Id;

  /** The raw natural-language request, exactly as the user wrote it. */
  rawText: string;

  /** Where the request came from. */
  source: IntentSource;

  /** When the intent was received. */
  createdAt: Timestamp;

  /** Current lifecycle status. */
  status: IntentStatus;

  /**
   * The structured objective cmdOS understood from rawText.
   * Undefined until status is at least "understood".
   */
  objective?: {
    /** A short, normalized summary of what to achieve. */
    summary: string;
    /** Optional structured parameters extracted from the request. */
    parameters?: Record<string, unknown>;
  };

  /** Populated only when status is "rejected": why it was rejected. */
  rejectionReason?: string;
}