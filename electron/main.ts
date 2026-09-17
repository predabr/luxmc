import { app, BrowserWindow, ipcMain, shell, dialog, protocol, net } from "electron";
import * as path from "path";
import * as fs from "fs";
import * as os from "os";
import { spawn, ChildProcess } from "child_process";
import * as readline from "readline";

protocol.registerSchemesAsPrivileged([
  {
    scheme: "app",
    privileges: {
      standard: true,
      secure: true,
      supportFetchAPI: true,
      corsEnabled: true,
      stream: true,
    },
  },
]);

let mainWindow: BrowserWindow | null = null;
let sidecarProcess: ChildProcess | null = null;
let nextRequestId = 1;
const pendingRequests = new Map<number, { resolve: (val: unknown) => void; reject: (err: unknown) => void }>();

if (process.defaultApp) {
  if (process.argv.length >= 2) {
    app.setAsDefaultProtocolClient("luxmc", process.execPath, [path.resolve(process.argv[1])]);
  }
} else {
  app.setAsDefaultProtocolClient("luxmc");
}

const gotSingleInstanceLock = app.requestSingleInstanceLock();
if (!gotSingleInstanceLock) {
  app.quit();
} else {
  app.on("second-instance", (_event, commandLine) => {
    if (mainWindow) {
      if (mainWindow.isMinimized()) mainWindow.restore();
      mainWindow.focus();
      const deepLink = commandLine.find((arg) => arg.startsWith("luxmc://"));
      if (deepLink) {
        mainWindow.webContents.send("deep-link", deepLink);
      }
    }
  });
}

function ensureExecutable(filePath: string) {
  if (process.platform !== "win32") {
    try {
      fs.chmodSync(filePath, 0o755);
    } catch {
      // Squashfs or read-only mount
    }
  }
}

function findSidecarBinary(): string | null {
  const isDev = !app.isPackaged;
  const binaryName = process.platform === "win32" ? "luxmc.exe" : "luxmc";

  if (isDev) {
    const devPaths = [
      path.join(__dirname, "../src-tauri/target/release", binaryName),
      path.join(__dirname, "../src-tauri/target/debug", binaryName),
      path.join(__dirname, "bin", binaryName),
    ];
    for (const p of devPaths) {
      if (fs.existsSync(p)) {
        ensureExecutable(p);
        return p;
      }
    }
  } else {
    const prodPaths = [
      path.join(process.resourcesPath, "bin", binaryName),
      path.join(process.resourcesPath, "app.asar.unpacked", "dist-electron", "bin", binaryName),
      path.join(process.resourcesPath, binaryName),
      path.join(path.dirname(app.getPath("exe")), "resources", "bin", binaryName),
      path.join(path.dirname(app.getPath("exe")), binaryName),
      path.join(os.homedir(), ".local/share/luxmc/bin", binaryName),
    ];
    for (const p of prodPaths) {
      if (fs.existsSync(p)) {
        ensureExecutable(p);
        return p;
      }
    }
  }

  return null;
}

function findCsharpLauncher(): string | null {
  const binaryName = process.platform === "win32" ? "Luxmc.Launcher.exe" : "Luxmc.Launcher";
  const isDev = !app.isPackaged;

  if (isDev) {
    const candidates = [
      path.join(__dirname, "../src-csharp/Luxmc.Launcher/bin/Release/net8.0/linux-x64/publish", binaryName),
      path.join(__dirname, "bin", binaryName),
      path.join(__dirname, "../dist-electron/bin", binaryName),
      path.join(os.homedir(), ".local/share/luxmc/bin", binaryName),
    ];
    for (const c of candidates) {
      if (fs.existsSync(c)) {
        ensureExecutable(c);
        return c;
      }
    }
  } else {
    const candidates = [
      path.join(process.resourcesPath, "bin", binaryName),
      path.join(process.resourcesPath, "app.asar.unpacked", "dist-electron", "bin", binaryName),
      path.join(process.resourcesPath, binaryName),
      path.join(path.dirname(app.getPath("exe")), "resources", "bin", binaryName),
      path.join(path.dirname(app.getPath("exe")), binaryName),
      path.join(os.homedir(), ".local/share/luxmc/bin", binaryName),
    ];
    for (const c of candidates) {
      if (fs.existsSync(c)) {
        ensureExecutable(c);
        return c;
      }
    }
  }
  return null;
}

function executeCsharpCommand(subcommand: string, inputArgs: string[]): Promise<any> {
  const launcherBin = findCsharpLauncher();
  if (!launcherBin) {
    return Promise.reject(new Error("C# Luxmc.Launcher binary not found"));
  }

  return new Promise((resolve, reject) => {
    const proc = spawn(launcherBin, [subcommand, ...inputArgs]);
    let out = "";
    let err = "";

    proc.stdout.on("data", (d) => { out += d.toString(); });
    proc.stderr.on("data", (d) => { err += d.toString(); });

    proc.on("close", (code) => {
      if (code === 0) {
        try {
          resolve(JSON.parse(out.trim()));
        } catch {
          resolve(out.trim());
        }
      } else {
        reject(new Error(err.trim() || out.trim() || `Exit code ${code}`));
      }
    });
  });
}

function startSidecar() {
  const binaryPath = findSidecarBinary();
  if (!binaryPath) {
    console.warn("[Luxmc Electron] Native sidecar binary not found, running in UI standalone mode.");
    return;
  }

  try {
    const csharpBin = findCsharpLauncher();
    sidecarProcess = spawn(binaryPath, ["--daemon"], {
      stdio: ["pipe", "pipe", "pipe"],
      env: {
        ...process.env,
        LUXMC_ELECTRON_MODE: "1",
        ...(csharpBin ? { LUXMC_CSHARP_PATH: csharpBin } : {}),
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
      mainWindow?.loadURL("app://luxmc/");
    });
  } else {
    mainWindow.loadURL("app://luxmc/");
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

  if (command === "csharp_validate" || command === "validate_instance") {
    const targetDir = (args?.path as string) || (args?.instanceDir as string) || process.cwd();
    return executeCsharpCommand("validate", [targetDir]);
  }

  if (command === "csharp_diagnose" || command === "diagnose_crash") {
    const logContent = (args?.log as string) || (args?.content as string) || "";
    return executeCsharpCommand("diagnose", [logContent]);
  }

  if (command === "csharp_gc_tune" || command === "gc_tune") {
    const ram = (args?.ramMb as number) || 4096;
    return executeCsharpCommand("gc-tune", [ram.toString()]);
  }

  if (command === "csharp_version") {
    return executeCsharpCommand("version", []);
  }

  // Forward command to the native Rust + C++ daemon
  return sendToSidecar(command, args);
});

app.whenReady().then(() => {
  protocol.handle("app", (req) => {
    const parsed = new URL(req.url);
    let pathname = decodeURIComponent(parsed.pathname);

    if (parsed.searchParams.has("path")) {
      const assetPath = parsed.searchParams.get("path");
      if (assetPath && fs.existsSync(assetPath)) {
        return net.fetch(`file://${assetPath}`);
      }
    }

    if (pathname.startsWith("/")) pathname = pathname.slice(1);
    const buildDir = path.join(__dirname, "../build");
    let target = path.join(buildDir, pathname || "index.html");
    if (!fs.existsSync(target) || (fs.existsSync(target) && fs.statSync(target).isDirectory())) {
      target = path.join(buildDir, "index.html");
    }
    return net.fetch(`file://${target}`);
  });

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
