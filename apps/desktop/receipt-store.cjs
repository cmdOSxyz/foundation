// Visibility: private
// apps/desktop/receipt-store.cjs
// Append-only execution receipts. Every real action Alios takes is recorded here:
// what it did, when, whether it was approved, the result, and if it's reversible.
// This is the audit trail — proof of exactly what happened. Records are never edited.

const { app } = require("electron");
const fs = require("node:fs");
const path = require("node:path");
const crypto = require("node:crypto");

function receiptFilePath() {
  return path.join(app.getPath("userData"), "cmdos-receipts.jsonl");
}

// Read all receipts (newest last). Each line is one JSON receipt.
function readReceipts() {
  try {
    const raw = fs.readFileSync(receiptFilePath(), "utf-8");
    return raw
      .split("\n")
      .filter((line) => line.trim())
      .map((line) => JSON.parse(line));
  } catch {
    return [];
  }
}

// Append one receipt. Returns the stored receipt (with id + hash).
function addReceipt(entry) {
  const receipt = {
    id: "rcpt-" + Date.now() + "-" + crypto.randomBytes(3).toString("hex"),
    timestamp: new Date().toISOString(),
    ...entry,
  };
  // A hash over the content makes tampering detectable.
  receipt.hash = crypto
    .createHash("sha256")
    .update(JSON.stringify({ ...receipt, hash: undefined }))
    .digest("hex")
    .slice(0, 16);

  fs.appendFileSync(receiptFilePath(), JSON.stringify(receipt) + "\n", "utf-8");
  return receipt;
}

module.exports = { addReceipt, readReceipts, receiptFilePath };