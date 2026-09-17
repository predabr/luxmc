import { contextBridge, ipcRenderer } from "electron";

export interface ElectronAPI {
  invoke: (command: string, args?: Record<string, unknown>) => Promise<unknown>;
  on: (channel: string, callback: (payload: unknown) => void) => () => void;
  window: {
    minimize: () => Promise<void>;
    maximize: () => Promise<void>;
    close: () => Promise<void>;
    isMaximized: () => Promise<boolean>;
  };
}

const electronAPI: ElectronAPI = {
  invoke: (command: string, args?: Record<string, unknown>) => {
    return ipcRenderer.invoke("luxmc:invoke", { command, args });
  },
  on: (channel: string, callback: (payload: unknown) => void) => {
    const listener = (_event: Electron.IpcRendererEvent, payload: unknown) => {
      callback(payload);
    };
    ipcRenderer.on(channel, listener);
    return () => {
      ipcRenderer.removeListener(channel, listener);
    };
  },
  window: {
    minimize: () => ipcRenderer.invoke("luxmc:window:minimize"),
    maximize: () => ipcRenderer.invoke("luxmc:window:maximize"),
    close: () => ipcRenderer.invoke("luxmc:window:close"),
    isMaximized: () => ipcRenderer.invoke("luxmc:window:isMaximized"),
  },
};

contextBridge.exposeInMainWorld("electronAPI", electronAPI);
