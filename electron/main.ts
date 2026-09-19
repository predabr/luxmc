import { app, BrowserWindow, ipcMain, shell, dialog, protocol, net } from "electron";
import * as path from "path";
import * as fs from "fs";
import * as os from "os";
import { spawn, ChildProcess } from "child_process";
import * as readline from "readline";
import { pathToFileURL } from "url";
import type { IpcMainInvokeEvent } from "electron";

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
const pendingRequests = new Map<number, { resolve: (val: unknown) => void; reject: (err: unknown) => void; timeout: ReturnType<typeof setTimeout> }>();

if (process.defaultApp) {
  if (process.argv.length >= 2) {
    app.setAsDefaultProtocolClient("luxmc", process.execPath, [path.resolve(process.argv[1])]);
  }
} else {
  app.setAsDefaultProtocolClient("luxmc");
}

const pendingDeepLinks: string[] = process.argv.filter(value => value.startsWith("luxmc://") && value.length <= 8192).slice(0, 32);
function receiveDeepLink(value: string): void {
  if (!value.startsWith("luxmc://") || value.length > 8192) return;
  if (pendingDeepLinks.length < 32 && !pendingDeepLinks.includes(value)) pendingDeepLinks.push(value);
  if (mainWindow) {
    mainWindow.show();
    if (mainWindow.isMinimized()) mainWindow.restore();
    mainWindow.focus();
    mainWindow.webContents.send("deep-link-pending", null);
  }
}
app.on("open-url", (event, url) => { event.preventDefault(); receiveDeepLink(url); });

const gotSingleInstanceLock = app.requestSingleInstanceLock();
if (!gotSingleInstanceLock) {
  app.quit();
} else {
  app.on("second-instance", (_event, commandLine) => {
    for (const value of commandLine) receiveDeepLink(value);
    mainWindow?.show();
    mainWindow?.focus();
  });
}

function trustedRenderer(value: string): boolean {
  try {
    const url = new URL(value);
    if (url.protocol === "app:" && url.hostname === "luxmc") return true;
    if (!app.isPackaged) {
      return url.origin === new URL(process.env.VITE_DEV_SERVER_URL || "http://localhost:1420").origin;
    }
  } catch {}
  return false;
}

function validateSender(event: IpcMainInvokeEvent) {
  if (!mainWindow || event.sender !== mainWindow.webContents || event.senderFrame !== mainWindow.webContents.mainFrame || !trustedRenderer(event.senderFrame.url)) {
    throw new Error("Untrusted IPC sender");
  }
}

function externalUrl(value: string): string {
  const url = new URL(value);
  if (!["https:", "http:"].includes(url.protocol) || url.username || url.password) throw new Error("Unsupported external URL");
  return url.toString();
}

function ensureExecutable(filePath: string) {
  if (process.platform !== "win32") {
    try {
      fs.chmodSync(filePath, 0o755);
    } catch {

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

function executeCsharpCommand(subcommand: string, inputArgs: string[]): Promise<unknown> {
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
            const { resolve, reject, timeout } = pendingRequests.get(msg.id)!;
            clearTimeout(timeout);
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
      for (const pending of pendingRequests.values()) {
        clearTimeout(pending.timeout);
        pending.reject(new Error("Native sidecar exited"));
      }
      pendingRequests.clear();
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

    const timeoutMs = /^(instance_import|instance_repair|modpack_update|mods_download_to_temp|launch_game|versions_download|loaders_install)/.test(command) ? 3600000 : 30000;
    const timeout = setTimeout(() => {
      if (pendingRequests.has(id)) {
        pendingRequests.delete(id);
        reject(new Error(`Command '${command}' timed out`));
      }
    }, timeoutMs);
    timeout.unref();
    pendingRequests.set(id, { resolve, reject, timeout });
    const payload = JSON.stringify({ id, command, args: args || {} }) + "\n";
    sidecarProcess!.stdin!.write(payload, (err) => {
      if (err) {
        clearTimeout(timeout);
        pendingRequests.delete(id);
        reject(err);
      }
    });


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
      sandbox: true,
    },
  });

  mainWindow.webContents.setWindowOpenHandler(({ url }) => {
    try { void shell.openExternal(externalUrl(url)).catch(() => {}); } catch {}
    return { action: "deny" };
  });
  mainWindow.webContents.on("will-navigate", (event, url) => {
    if (!trustedRenderer(url)) {
      event.preventDefault();
      try { void shell.openExternal(externalUrl(url)).catch(() => {}); } catch {}
    }
  });
  mainWindow.webContents.session.setPermissionRequestHandler((_contents, _permission, callback) => callback(false));
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

ipcMain.handle("luxmc:window:minimize", (event) => {
  validateSender(event);
  mainWindow?.minimize();
});

ipcMain.handle("luxmc:window:maximize", (event) => {
  validateSender(event);
  if (mainWindow?.isMaximized()) {
    mainWindow.unmaximize();
  } else {
    mainWindow?.maximize();
  }
});

ipcMain.handle("luxmc:window:close", (event) => {
  validateSender(event);
  mainWindow?.close();
});

ipcMain.handle("luxmc:window:isMaximized", (event) => {
  validateSender(event);
  return mainWindow?.isMaximized() ?? false;
});

ipcMain.handle("luxmc:invoke", async (event, { command, args }: { command: string; args?: Record<string, unknown> }) => {
  validateSender(event);

  if (command === "deep_links_take") return pendingDeepLinks.splice(0);
  if (command === "open_portal") {
    const paths: Record<string, string> = { home: "", catalog: "#mods", news: "#changelog", skins: "skins.html" };
    if (typeof args?.section !== "string" || !Object.hasOwn(paths, args.section)) throw new Error("Página do portal inválida");
    await shell.openExternal(`https://luxmc-r92.pages.dev/${paths[args.section]}`);
    return;
  }
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
      await shell.openExternal(externalUrl(url));
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

  return sendToSidecar(command, args);
});

app.whenReady().then(() => {
  protocol.handle("app", (req) => {
    try {
      const parsed = new URL(req.url);
      if (parsed.hostname !== "luxmc") return new Response("Forbidden", { status: 403 });
      if (parsed.pathname === "/asset") {
        const assetPath = parsed.searchParams.get("path");
        if (!assetPath || !path.isAbsolute(assetPath)) return new Response("Invalid asset", { status: 400 });
        const canonical = fs.realpathSync(assetPath);
        if (!/\.(png|jpe?g|webp|gif|bmp|ico|mp3|ogg|wav|mp4|webm)$/i.test(canonical) || !fs.statSync(canonical).isFile()) return new Response("Forbidden", { status: 403 });
        return net.fetch(pathToFileURL(canonical).href);
      }
      const pathname = decodeURIComponent(parsed.pathname);
      const buildDir = fs.realpathSync(path.join(__dirname, "../build"));
      let target = path.resolve(buildDir, "." + pathname);
      const relative = path.relative(buildDir, target);
      if (relative.startsWith("..") || path.isAbsolute(relative)) return new Response("Forbidden", { status: 403 });
      if (!fs.existsSync(target) || fs.statSync(target).isDirectory()) target = path.join(buildDir, "index.html");
      const canonical = fs.realpathSync(target);
      const resolved = path.relative(buildDir, canonical);
      if (resolved.startsWith("..") || path.isAbsolute(resolved)) return new Response("Forbidden", { status: 403 });
      return net.fetch(pathToFileURL(canonical).href);
    } catch { return new Response("Not found", { status: 404 }); }
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
