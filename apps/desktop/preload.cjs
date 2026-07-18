// Visibility: private
// apps/desktop/preload.cjs
// The secure bridge. Exposes only these explicit functions to the UI.

const { contextBridge, ipcRenderer } = require("electron");

contextBridge.exposeInMainWorld("cmdos", {
  ping: (message) => ipcRenderer.invoke("cmdos:ping", message),
  // Ask main to run one approved filesystem step. Returns { ok, message }.
  runStep: (step) => ipcRenderer.invoke("cmdos:runStep", step),
});