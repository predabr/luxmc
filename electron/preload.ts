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

contextBridge.exposeInMainWorld("__TAURI_INTERNALS__", {
  invoke: (cmd: string, args?: Record<string, unknown>) => {
    if (cmd === "plugin:app|version") {
      return Promise.resolve("1.7.6");
    }
    if (cmd === "plugin:event|listen") {
      return Promise.resolve(1);
    }
    if (cmd === "plugin:event|unlisten") {
      return Promise.resolve();
    }
    if (cmd === "plugin:store|load") {
      return Promise.resolve(1);
    }
    if (cmd === "plugin:store|get") {
      try {
        const raw = localStorage.getItem(`luxmc_store_${args?.key || "app"}`);
        if (raw !== null) {
          return Promise.resolve([JSON.parse(raw), true]);
        }
      } catch {}
      return Promise.resolve([null, false]);
    }
    if (cmd === "plugin:store|set") {
      try {
        localStorage.setItem(`luxmc_store_${args?.key || "app"}`, JSON.stringify(args?.value));
      } catch {}
      return Promise.resolve();
    }
    if (cmd === "plugin:store|save") {
      return Promise.resolve();
    }
    return ipcRenderer.invoke("luxmc:invoke", { command: cmd, args });
  },
  transformCallback: (callback: (payload: unknown) => void) => callback,
  convertFileSrc: (filePath: string) => {
    if (!filePath) return "";
    return `app://luxmc/asset?path=${encodeURIComponent(filePath)}`;
  },
  metadata: {
    currentWindow: { label: "main" },
  },
});
