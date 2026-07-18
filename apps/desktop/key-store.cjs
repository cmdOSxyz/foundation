// Visibility: private
// apps/desktop/key-store.cjs
// Stores the user's API key in the app's local user-data folder.
// The key never leaves the user's machine and is never committed to git.

const { app } = require("electron");
const fs = require("node:fs");
const path = require("node:path");

// A file inside the OS-managed app data folder (e.g. %APPDATA%/cmdOS on Windows).
function keyFilePath() {
  return path.join(app.getPath("userData"), "cmdos-keys.json");
}

/** Read all stored keys as an object, e.g. { anthropic: "sk-ant-..." }. */
function readKeys() {
  try {
    const raw = fs.readFileSync(keyFilePath(), "utf-8");
    return JSON.parse(raw);
  } catch {
    return {}; // no file yet
  }
}

/** Save one provider's key. */
function setKey(provider, key) {
  const keys = readKeys();
  keys[provider] = key;
  fs.writeFileSync(keyFilePath(), JSON.stringify(keys, null, 2), "utf-8");
  return true;
}

/** Get one provider's key, or null. */
function getKey(provider) {
  const keys = readKeys();
  return keys[provider] || null;
}

/** Report whether a provider has a key, without exposing the key itself. */
function hasKey(provider) {
  return Boolean(getKey(provider));
}

module.exports = { setKey, getKey, hasKey, keyFilePath };