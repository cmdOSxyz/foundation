// Visibility: private
// apps/desktop/preload.cjs
// The secure bridge. It exposes a small, explicit API to the renderer.
// The UI can ONLY call what we list here — it has no direct system access.

const { contextBridge, ipcRenderer } = require("electron");

contextBridge.exposeInMainWorld("cmdos", {
  // Test call: renderer asks main to echo a message back.
  ping: (message) => ipcRenderer.invoke("cmdos:ping", message),
});