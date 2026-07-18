// Visibility: private
// apps/desktop/main.js
// Electron main process. Creates the cmdOS application window.

const path = require("node:path");
const { app, BrowserWindow, ipcMain } = require("electron");

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