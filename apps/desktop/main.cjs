// Visibility: private
// apps/desktop/main.js
// Electron main process. Creates the cmdOS application window.
// Load the compiled core (built into dist/ by `npm run build`).

const { planWithClaude } = require("./anthropic-planner.cjs");
const { runFilesystemStep, verifyFilesystemStep, inspectPath } = require("../../dist/capabilities/filesystem.js");
const path = require("node:path");
const { app, BrowserWindow, ipcMain } = require("electron");
const keyStore = require("./key-store.cjs");

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
    return { ok: check.ok, message: result.message + " | " + check.message };
  } catch (err) {
    return { ok: false, message: "FAILED: " + (err && err.message ? err.message : String(err)) };
  }
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