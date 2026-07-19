// Visibility: private
// apps/desktop/preload.cjs
const { contextBridge, ipcRenderer } = require("electron");

contextBridge.exposeInMainWorld("cmdos", {
  ping: (message) => ipcRenderer.invoke("cmdos:ping", message),
  runStep: (step) => ipcRenderer.invoke("cmdos:runStep", step),
  setKey: (provider, key) => ipcRenderer.invoke("cmdos:setKey", provider, key),
  hasKey: (provider) => ipcRenderer.invoke("cmdos:hasKey", provider),
  plan: (intentText, history) => ipcRenderer.invoke("cmdos:plan", intentText, history),
  pickWorkspace: () => ipcRenderer.invoke("cmdos:pickWorkspace"),
  getWorkspace: () => ipcRenderer.invoke("cmdos:getWorkspace"),
});