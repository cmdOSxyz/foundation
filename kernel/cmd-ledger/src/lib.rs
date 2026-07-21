//! # cmd-ledger — the append-only, hash-chained audit ledger
//!
//! Every action, score, and intervention in cmdOS is recorded here. Once an
//! entry is appended it is immutable: entries can be added and read, never
//! modified or deleted. This is the observability and audit foundation of the
//! whole runtime, and the user's proof of exactly what their agent did.
//!
//! Reference behavior comes from two prototype files:
//! - `prototype/kernel/event-log.ts` — append-only, immutable, read-returns-copy.
//! - `prototype/apps/desktop/receipt-store.cjs` — adds a content hash so
//!   tampering is detectable.
//!
//! `cmd-ledger` combines both and goes one step further: entries are **hash
//! chained**. Each entry's hash covers the previous entry's hash, so altering
//! any past entry breaks every hash after it. This is what makes the ledger
//! "signed" in spirit — the chain is self-verifying.

use cmd_types::Event;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// The genesis hash: the "previous hash" of the very first entry.
const GENESIS: &str = "0000000000000000000000000000000000000000000000000000000000000000";

/// One immutable record in the ledger: an [`Event`] plus its position in the
/// hash chain.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LedgerEntry {
    /// Zero-based position in the chain.
    pub index: u64,
    /// Hash of the previous entry (or [`GENESIS`] for the first).
    pub prev_hash: String,
    /// Hash over `index`, `prev_hash`, and the event's JSON. Hex, 64 chars.
    pub hash: String,
    /// The recorded event.
    pub event: Event,
}

impl LedgerEntry {
    /// Recompute the hash this entry *should* have, from its own contents.
    /// Used to verify integrity.
    fn compute_hash(index: u64, prev_hash: &str, event: &Event) -> String {
        // Deterministic: Event serializes with BTreeMaps, so the JSON is stable.
        let event_json = serde_json::to_string(event).expect("Event serializes");
        let mut hasher = Sha256::new();
        hasher.update(index.to_le_bytes());
        hasher.update(prev_hash.as_bytes());
        hasher.update(event_json.as_bytes());
        hex(&hasher.finalize())
    }
}

/// Encode bytes as lowercase hex.
fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        let _ = write!(s, "{b:02x}");
    }
    s
}

/// An append-only, hash-chained ledger held in memory.
///
/// Persistence (writing the chain to disk / SemFS) is a later concern; this type
/// owns the integrity model. Nothing here exposes a way to mutate or remove a
/// stored entry.
#[derive(Debug, Default)]
pub struct Ledger {
    entries: Vec<LedgerEntry>,
}

impl Ledger {
    /// A new, empty ledger.
    pub fn new() -> Self {
        Ledger {
            entries: Vec::new(),
        }
    }

    /// Append one event. Returns a clone of the stored entry (callers can never
    /// reach into the internal store to rewrite history).
    pub fn append(&mut self, event: Event) -> LedgerEntry {
        let index = self.entries.len() as u64;
        let prev_hash = self
            .entries
            .last()
            .map(|e| e.hash.clone())
            .unwrap_or_else(|| GENESIS.to_string());
        let hash = LedgerEntry::compute_hash(index, &prev_hash, &event);
        let entry = LedgerEntry {
            index,
            prev_hash,
            hash,
            event,
        };
        self.entries.push(entry.clone());
        entry
    }

    /// All entries, oldest first, as a copy of the internal store.
    pub fn all(&self) -> Vec<LedgerEntry> {
        self.entries.clone()
    }

    /// Number of entries recorded.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the ledger has no entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Entries whose event is of the given type.
    pub fn by_type(&self, event_type: cmd_types::EventType) -> Vec<LedgerEntry> {
        self.entries
            .iter()
            .filter(|e| e.event.event_type == event_type)
            .cloned()
            .collect()
    }

    /// Entries related to a given plan id.
    pub fn by_plan(&self, plan_id: cmd_types::Id) -> Vec<LedgerEntry> {
        self.entries
            .iter()
            .filter(|e| e.event.plan_id == Some(plan_id))
            .cloned()
            .collect()
    }

    /// Verify the whole chain: every entry's hash recomputes correctly and links
    /// to the previous one. Returns `Ok(())` if intact, or the index of the
    /// first broken entry. This is how tampering is detected.
    pub fn verify(&self) -> Result<(), LedgerError> {
        let mut expected_prev = GENESIS.to_string();
        for (i, entry) in self.entries.iter().enumerate() {
            if entry.index != i as u64 {
                return Err(LedgerError::BrokenAt(i as u64));
            }
            if entry.prev_hash != expected_prev {
                return Err(LedgerError::BrokenAt(i as u64));
            }
            let recomputed = LedgerEntry::compute_hash(entry.index, &entry.prev_hash, &entry.event);
            if recomputed != entry.hash {
                return Err(LedgerError::BrokenAt(i as u64));
            }
            expected_prev = entry.hash.clone();
        }
        Ok(())
    }
}

/// Errors the ledger can report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LedgerError {
    /// The chain is broken starting at this entry index (tampering or corruption).
    BrokenAt(u64),
}

impl std::fmt::Display for LedgerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LedgerError::BrokenAt(i) => write!(f, "ledger integrity broken at entry {i}"),
        }
    }
}

impl std::error::Error for LedgerError {}

#[cfg(test)]
mod tests {
    use super::*;
    use cmd_types::{Event, EventType, Id};

    #[test]
    fn append_assigns_sequential_indexes_and_links_hashes() {
        let mut ledger = Ledger::new();
        let a = ledger.append(Event::new(EventType::IntentReceived));
        let b = ledger.append(Event::new(EventType::PlanCreated));

        assert_eq!(a.index, 0);
        assert_eq!(a.prev_hash, GENESIS);
        assert_eq!(b.index, 1);
        // b links to a.
        assert_eq!(b.prev_hash, a.hash);
        assert_ne!(a.hash, b.hash);
    }

    #[test]
    fn empty_ledger_reports_empty() {
        let ledger = Ledger::new();
        assert!(ledger.is_empty());
        assert_eq!(ledger.len(), 0);
        // An empty chain is trivially intact.
        assert!(ledger.verify().is_ok());
    }

    #[test]
    fn intact_chain_verifies() {
        let mut ledger = Ledger::new();
        for _ in 0..5 {
            ledger.append(Event::new(EventType::StepStarted));
        }
        assert_eq!(ledger.len(), 5);
        assert!(ledger.verify().is_ok());
    }

    #[test]
    fn tampering_is_detected() {
        let mut ledger = Ledger::new();
        ledger.append(Event::new(EventType::IntentReceived));
        ledger.append(Event::new(EventType::PlanCreated));
        ledger.append(Event::new(EventType::StepSucceeded));

        // Forge history: change a stored event but leave its hash alone.
        ledger.entries[1].event.event_type = EventType::TransactionRolledBack;

        // The chain must notice the entry no longer hashes to its stored hash.
        assert_eq!(ledger.verify(), Err(LedgerError::BrokenAt(1)));
    }

    #[test]
    fn reads_return_copies_not_internal_state() {
        let mut ledger = Ledger::new();
        ledger.append(Event::new(EventType::IntentReceived));
        let mut snapshot = ledger.all();
        // Mutating the returned copy must not affect the ledger.
        snapshot[0].event.event_type = EventType::SupervisorIntervened;
        assert!(ledger.verify().is_ok());
        assert_eq!(ledger.all()[0].event.event_type, EventType::IntentReceived);
    }

    #[test]
    fn filters_by_type_and_plan() {
        let mut ledger = Ledger::new();
        let plan = Id::new();

        let mut e1 = Event::new(EventType::PlanCreated);
        e1.plan_id = Some(plan);
        ledger.append(e1);

        ledger.append(Event::new(EventType::IntentReceived)); // no plan

        let mut e3 = Event::new(EventType::StepSucceeded);
        e3.plan_id = Some(plan);
        ledger.append(e3);

        assert_eq!(ledger.by_plan(plan).len(), 2);
        assert_eq!(ledger.by_type(EventType::IntentReceived).len(), 1);
    }
}
