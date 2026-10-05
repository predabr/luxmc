import { spawnSync } from 'node:child_process';
import { copyFileSync, existsSync, mkdirSync, chmodSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { homedir } from 'node:os';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const packageManager = process.env.npm_execpath;
if (!packageManager) throw new Error('Execute pnpm electron:build.');
const windows = process.platform === 'win32';
const rustDirectory = join(homedir(), '.cargo', 'bin');
if (windows && existsSync(join(rustDirectory, 'cargo.exe'))) process.env.PATH = `${rustDirectory};${process.env.PATH ?? ''}`;
if (!windows && process.platform !== 'linux') throw new Error('Este empacotador suporta Windows e Linux.');
const binaryName = windows ? 'luxmc.exe' : 'luxmc';
const runtimeId = `${windows ? 'win' : 'linux'}-${process.arch === 'arm64' ? 'arm64' : 'x64'}`;
const binDir = join(root, 'dist-electron', 'bin');
function run(command, args, cwd = root) {
    const result = spawnSync(command, args, { cwd, stdio: 'inherit', windowsHide: true });
    if (result.error) throw result.error;
    if (result.status !== 0) throw new Error(`Falha ao executar ${command} (${result.status}).`);
}
console.log(`Compilando Luxmc para ${runtimeId}`);
run('cargo', ['build', '--release', '--locked'], join(root, 'src-tauri'));
run(process.execPath, [packageManager, 'build']);
run(process.execPath, ['electron/build.js']);
mkdirSync(binDir, { recursive: true });
const sidecar = join(root, 'src-tauri', 'target', 'release', binaryName);
if (!existsSync(sidecar)) throw new Error(`Motor nativo não encontrado: ${sidecar}`);
copyFileSync(sidecar, join(binDir, binaryName));
if (!windows) chmodSync(join(binDir, binaryName), 0o755);
if (process.env.LUXMC_BUILD_CSHARP === '1') {
    run('dotnet', ['publish', 'src-csharp/Luxmc.Launcher/Luxmc.Launcher.csproj', '-c', 'Release', '-r', runtimeId, '--self-contained', '-p:PublishSingleFile=true']);
    const csharpName = windows ? 'Luxmc.Launcher.exe' : 'Luxmc.Launcher';
    const csharp = join(root, 'src-csharp', 'Luxmc.Launcher', 'bin', 'Release', 'net8.0', runtimeId, 'publish', csharpName);
    copyFileSync(csharp, join(binDir, csharpName));
    if (!windows) chmodSync(join(binDir, csharpName), 0o755);
}
run(process.execPath, ['node_modules/electron-builder/cli.js', windows ? '--win' : '--linux', windows ? 'nsis' : 'AppImage']);
console.log('Instalador gerado em release-electron.');
