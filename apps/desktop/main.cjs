// Visibility: private
// apps/desktop/main.js
// Electron main process. Creates the cmdOS application window.
// Load the compiled core (built into dist/ by `npm run build`).

const { planWithClaude } = require("./anthropic-planner.cjs");
const { runFilesystemStep, verifyFilesystemStep, inspectPath, dryRunFilesystemStep } = require("../../dist/capabilities/filesystem.js");
const path = require("node:path");
const { app, BrowserWindow, ipcMain, dialog } = require("electron");
const keyStore = require("./key-store.cjs");
const receiptStore = require("./receipt-store.cjs");

// Remembers the last undoable action so the UI can reverse it.
let lastUndoable = null;

function createWindow() {
  const win = new BrowserWindow({
    width: 1200,
    height: 800,
    backgroundColor: "#0a0a0f", // dark, per cmdOS UI direction
    title: "cmdOS",
    webPreferences: {
      // Security defaults: no direct Node access from the UI yet.
      contextIsolation: true,
      nodeIntegration: false,
      preload: path.join(__dirname, "preload.cjs"),
    },
  });

  // Load the UI. For now, a simple local HTML file.
  win.loadFile(path.join(__dirname, "index.html"));
}
ipcMain.handle("cmdos:ping", async (event, message) => {
  return "pong: " + message + " (from main process)";
});

// Run one approved filesystem step, then verify it.
ipcMain.handle("cmdos:runStep", async (event, step) => {
  try {
    const result = await runFilesystemStep(step);
    const check = await verifyFilesystemStep(step);

    // Record how to undo this action, if possible.
    lastUndoable = null;
    if (step.action === "rename" || step.action === "move") {
      const data = result.data || {};
      if (data.from && data.to) {
        lastUndoable = { action: step.action, from: data.to, to: data.from, label: step.action };
      }
    } else if (step.action === "delete") {
      const data = result.data || {};
      if (data.trashedPath && data.path) {
        lastUndoable = { action: "restore", from: data.trashedPath, to: data.path, label: "delete" };
      }
    }

    // Record an immutable execution receipt (proof of what happened).
    receiptStore.addReceipt({
      capability: step.capability,
      action: step.action,
      description: step.description || "",
      parameters: step.parameters || {},
      approved: Boolean(step.requiresPermission),
      result: check.ok ? "success" : "failed",
      message: result.message,
      reversible: Boolean(lastUndoable),
    });

    return { ok: check.ok, message: result.message + " | " + check.message, canUndo: Boolean(lastUndoable) };
  } catch (err) {
    return { ok: false, message: "FAILED: " + (err && err.message ? err.message : String(err)), canUndo: false };
  }
});

// Undo the last undoable action.
ipcMain.handle("cmdos:undo", async () => {
  if (!lastUndoable) return { ok: false, message: "Nothing to undo" };
  try {
    const { rename } = require("node:fs/promises");
    const { resolve } = require("node:path");
    await rename(resolve(lastUndoable.from), resolve(lastUndoable.to));
    const label = lastUndoable.label;
    lastUndoable = null;
    return { ok: true, message: "Undone: " + label };
  } catch (err) {
    return { ok: false, message: "Undo failed: " + (err && err.message ? err.message : String(err)) };
  }
});

ipcMain.handle("cmdos:getReceipts", async () => {
  const all = receiptStore.readReceipts();
  return { ok: true, receipts: all.reverse() };
});

// Save the user's API key (BYOK). Never logged, never sent anywhere.
ipcMain.handle("cmdos:setKey", async (event, provider, key) => {
  keyStore.setKey(provider, key);
  return { ok: true };
});

// Report whether a provider key is present (does NOT return the key).
ipcMain.handle("cmdos:hasKey", async (event, provider) => {
  return { ok: true, hasKey: keyStore.hasKey(provider) };
});

// Two-pass planning: first find out what to inspect, gather real facts,
// then ask the agent to reply and plan based on those facts.
ipcMain.handle("cmdos:plan", async (event, intentText, history) => {
  const apiKey = keyStore.getKey("anthropic");
  if (!apiKey) return { ok: false, message: "No Claude API key set" };
  try {
    const result = await planWithClaude(apiKey, intentText, inspectPath, history);
    return { ok: true, plan: result };
  } catch (err) {
    return { ok: false, message: err && err.message ? err.message : String(err) };
  }
});

// Let the user pick a workspace folder.
ipcMain.handle("cmdos:pickWorkspace", async () => {
  const result = await dialog.showOpenDialog({
    title: "Select a workspace folder for Alios",
    properties: ["openDirectory"],
  });
  if (result.canceled || !result.filePaths[0]) return { ok: false };
  const dir = result.filePaths[0];
  keyStore.setKey("workspace", dir); // reuse key-store to persist it
  return { ok: true, path: dir };
});

// Return the current workspace path (or null).
ipcMain.handle("cmdos:getWorkspace", async () => {
  return { ok: true, path: keyStore.getKey("workspace") };
});

// Preview a step's effect without executing it.
ipcMain.handle("cmdos:dryRun", async (event, step) => {
  try {
    const preview = await dryRunFilesystemStep(step);
    return { ok: true, preview };
  } catch (err) {
    return { ok: false, message: err && err.message ? err.message : String(err) };
  }
});

app.whenReady().then(() => {
  createWindow();

  // On macOS, re-open a window when the dock icon is clicked.
  app.on("activate", () => {
    if (BrowserWindow.getAllWindows().length === 0) createWindow();
  });
});

// Quit when all windows are closed (except on macOS).
app.on("window-all-closed", () => {
  if (process.platform !== "darwin") app.quit();
});