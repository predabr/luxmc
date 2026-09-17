import { app, BrowserWindow, ipcMain, shell, dialog } from "electron";
import * as path from "path";
import * as fs from "fs";
import { spawn, ChildProcess } from "child_process";
import * as readline from "readline";

let mainWindow: BrowserWindow | null = null;
let sidecarProcess: ChildProcess | null = null;
let nextRequestId = 1;
const pendingRequests = new Map<number, { resolve: (val: unknown) => void; reject: (err: unknown) => void }>();

function findSidecarBinary(): string | null {
  const isDev = !app.isPackaged;
  const binaryName = process.platform === "win32" ? "luxmc.exe" : "luxmc";

  if (isDev) {
    const devPaths = [
      path.join(__dirname, "../src-tauri/target/release", binaryName),
      path.join(__dirname, "../src-tauri/target/debug", binaryName),
    ];
    for (const p of devPaths) {
      if (fs.existsSync(p)) return p;
    }
  } else {
    const prodPaths = [
      path.join(__dirname, "bin", binaryName),
      path.join(process.resourcesPath, "bin", binaryName),
      path.join(process.resourcesPath, binaryName),
      path.join(path.dirname(app.getPath("exe")), binaryName),
    ];
    for (const p of prodPaths) {
      if (fs.existsSync(p)) return p;
    }
  }

  return null;
}

function startSidecar() {
  const binaryPath = findSidecarBinary();
  if (!binaryPath) {
    console.warn("[Luxmc Electron] Native sidecar binary not found, running in UI standalone mode.");
    return;
  }

  try {
    sidecarProcess = spawn(binaryPath, ["--daemon"], {
      stdio: ["pipe", "pipe", "pipe"],
      env: {
        ...process.env,
        LUXMC_ELECTRON_MODE: "1",
      },
    });

    if (sidecarProcess.stdout) {
      const rl = readline.createInterface({ input: sidecarProcess.stdout });
      rl.on("line", (line) => {
        try {
          const msg = JSON.parse(line);
          if (msg.id && pendingRequests.has(msg.id)) {
            const { resolve, reject } = pendingRequests.get(msg.id)!;
            pendingRequests.delete(msg.id);
            if (msg.error) {
              reject(new Error(msg.error));
            } else {
              resolve(msg.result);
            }
          } else if (msg.event && mainWindow && !mainWindow.isDestroyed()) {
            mainWindow.webContents.send(msg.event, msg.payload);
          }
        } catch {
          // Non-JSON output from sidecar
        }
      });
    }

    if (sidecarProcess.stderr) {
      sidecarProcess.stderr.on("data", (chunk) => {
        console.error(`[Sidecar stderr] ${chunk.toString()}`);
      });
    }

    sidecarProcess.on("exit", (code) => {
      console.warn(`[Luxmc Electron] Native sidecar exited with code ${code}`);
      sidecarProcess = null;
    });
  } catch (err) {
    console.error("[Luxmc Electron] Failed to spawn native sidecar:", err);
  }
}

function sendToSidecar(command: string, args?: Record<string, unknown>): Promise<unknown> {
  if (!sidecarProcess || !sidecarProcess.stdin || sidecarProcess.stdin.destroyed) {
    return Promise.reject(new Error(`Native sidecar is not running for command: ${command}`));
  }

  return new Promise((resolve, reject) => {
    const id = nextRequestId++;
    pendingRequests.set(id, { resolve, reject });

    const payload = JSON.stringify({ id, command, args: args || {} }) + "\n";
    sidecarProcess!.stdin!.write(payload, (err) => {
      if (err) {
        pendingRequests.delete(id);
        reject(err);
      }
    });

    setTimeout(() => {
      if (pendingRequests.has(id)) {
        pendingRequests.delete(id);
        reject(new Error(`Command '${command}' timed out after 30s`));
      }
    }, 30000);
  });
}

function createWindow() {
  mainWindow = new BrowserWindow({
    title: "Luxmc",
    width: 1180,
    height: 760,
    minWidth: 960,
    minHeight: 600,
    backgroundColor: "#0c0c0e",
    show: false,
    frame: true,
    autoHideMenuBar: true,
    webPreferences: {
      preload: path.join(__dirname, "preload.cjs"),
      contextIsolation: true,
      nodeIntegration: false,
      sandbox: false,
    },
  });

  mainWindow.maximize();

  mainWindow.once("ready-to-show", () => {
    if (mainWindow) {
      mainWindow.show();
    }
  });

  mainWindow.on("closed", () => {
    mainWindow = null;
  });

  const devUrl = process.env.VITE_DEV_SERVER_URL || "http://localhost:1420";
  const isDev = !app.isPackaged;

  if (isDev) {
    mainWindow.loadURL(devUrl).catch(() => {
      const prodPath = path.join(__dirname, "../build/index.html");
      if (fs.existsSync(prodPath) && mainWindow) {
        mainWindow.loadFile(prodPath);
      }
    });
  } else {
    const prodPath = path.join(__dirname, "../build/index.html");
    mainWindow.loadFile(prodPath);
  }
}

// Window Controls IPC
ipcMain.handle("luxmc:window:minimize", () => {
  mainWindow?.minimize();
});

ipcMain.handle("luxmc:window:maximize", () => {
  if (mainWindow?.isMaximized()) {
    mainWindow.unmaximize();
  } else {
    mainWindow?.maximize();
  }
});

ipcMain.handle("luxmc:window:close", () => {
  mainWindow?.close();
});

ipcMain.handle("luxmc:window:isMaximized", () => {
  return mainWindow?.isMaximized() ?? false;
});

// Main Invoke Handler (Native sidecar + Node fallback)
ipcMain.handle("luxmc:invoke", async (_event, { command, args }: { command: string; args?: Record<string, unknown> }) => {
  // Built-in system commands handled natively by Electron
  if (command === "ping") {
    return "pong";
  }

  if (command === "open_folder" || command === "instances_open_folder" || command === "screenshots_open_folder") {
    const targetPath = (args?.path as string) || (args?.instancePath as string);
    if (targetPath) {
      await shell.openPath(targetPath);
      return true;
    }
  }

  if (command === "open_url") {
    const url = args?.url as string;
    if (url) {
      await shell.openExternal(url);
      return true;
    }
  }

  if (command === "select_folder" || command === "dialog_open_dir") {
    if (mainWindow) {
      const res = await dialog.showOpenDialog(mainWindow, {
        properties: ["openDirectory"],
      });
      return res.filePaths[0] || null;
    }
    return null;
  }

  if (command === "select_file" || command === "dialog_open_file") {
    if (mainWindow) {
      const res = await dialog.showOpenDialog(mainWindow, {
        properties: ["openFile"],
      });
      return res.filePaths[0] || null;
    }
    return null;
  }

  // Forward command to the native Rust + C++ daemon
  return sendToSidecar(command, args);
});

app.whenReady().then(() => {
  startSidecar();
  createWindow();

  app.on("activate", () => {
    if (BrowserWindow.getAllWindows().length === 0) {
      createWindow();
    }
  });
});

app.on("before-quit", () => {
  if (sidecarProcess && !sidecarProcess.killed) {
    sidecarProcess.kill("SIGTERM");
    sidecarProcess = null;
  }
});

app.on("window-all-closed", () => {
  if (process.platform !== "darwin") {
    app.quit();
  }
});
