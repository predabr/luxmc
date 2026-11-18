$ErrorActionPreference = 'Stop'
$projectDirectory = 'C:/Users/preda/OneDrive/Documenti/Luxmc'
$installDirectory = [IO.Path]::GetFullPath((Join-Path $env:LOCALAPPDATA 'Luxmc'))
$validationDirectory = Join-Path $projectDirectory 'docs/validation/launch-update-plans'
$payloadDirectory = Join-Path $validationDirectory 'payload'
$registryPath = 'HKCU:/Software/Microsoft/Windows/CurrentVersion/Uninstall/Luxmc'
$registered = Get-ItemProperty -LiteralPath $registryPath
if ([IO.Path]::GetFullPath($registered.InstallLocation.Trim('"')) -ne $installDirectory) { throw 'Unexpected installed application directory.' }
if (Get-Process java,javaw,luxmc -ErrorAction SilentlyContinue) { throw 'Feche o Minecraft e o Luxmc antes de atualizar os arquivos.' }
$audit = Get-Content -LiteralPath (Join-Path $validationDirectory 'windows-audit.json') -Raw | ConvertFrom-Json
if ($audit.defender.scanExitCode -ne 0) { throw 'Defender validation was not completed.' }
$program = Join-Path $payloadDirectory 'luxmc.exe'
$uninstaller = Join-Path $payloadDirectory 'uninstall.exe'
$installer = Join-Path $projectDirectory 'release-windows/Lux MC Launcher - Inicio Seguro.exe'
$programReceipt = $audit.components | Where-Object component -eq 'program'
$installerReceipt = $audit.components | Where-Object component -eq 'installer'
$uninstallerReceipt = $audit.components | Where-Object component -eq 'uninstaller'
if ((Get-FileHash -LiteralPath $program -Algorithm SHA256).Hash.ToLowerInvariant() -ne $programReceipt.sha256) { throw 'Program changed after validation.' }
if ((Get-FileHash -LiteralPath $installer -Algorithm SHA256).Hash.ToLowerInvariant() -ne $installerReceipt.sha256) { throw 'Installer changed after validation.' }
if (!(Test-Path -LiteralPath $uninstaller) -or (Get-Item -LiteralPath $uninstaller).Length -lt 65536) { throw 'Extracted uninstaller is missing or incomplete.' }
if (!$uninstallerReceipt -or (Get-FileHash -LiteralPath $uninstaller -Algorithm SHA256).Hash.ToLowerInvariant() -ne $uninstallerReceipt.sha256) { throw 'Uninstaller changed after validation.' }
$protectedFiles = @(
    (Join-Path $env:APPDATA 'github/Luxmc/data/luxmc.db'),
    (Join-Path $env:APPDATA 'github/Luxmc/data/luxmc.db-wal'),
    (Join-Path $env:APPDATA 'github/Luxmc/data/luxmc.db-shm'),
    (Join-Path $env:APPDATA 'io.github.luxmc.Luxmc/auth.json'),
    (Join-Path $env:APPDATA 'io.github.luxmc.Luxmc/settings.json')
)
$backupDirectory = Join-Path $env:LOCALAPPDATA ('Luxmc-build/backups/inicio-seguro-' + (Get-Date -Format 'yyyyMMdd-HHmmss'))
New-Item -ItemType Directory -Path $backupDirectory -Force | Out-Null
$beforeHashes = @{}
for ($index=0; $index -lt $protectedFiles.Count; $index++) {
    $file = $protectedFiles[$index]
    if (Test-Path -LiteralPath $file) {
        $beforeHashes[$file] = (Get-FileHash -LiteralPath $file -Algorithm SHA256).Hash
        Copy-Item -LiteralPath $file -Destination (Join-Path $backupDirectory ("data-$index-" + [IO.Path]::GetFileName($file)))
    }
}
foreach ($name in @('luxmc.exe','luxmc-repair.exe','uninstall.exe')) {
    $file = Join-Path $installDirectory $name
    if (Test-Path -LiteralPath $file) { Copy-Item -LiteralPath $file -Destination (Join-Path $backupDirectory $name) }
}
$shell = New-Object -ComObject WScript.Shell
$shortcuts = @{}
foreach ($root in @([Environment]::GetFolderPath('Desktop'),[Environment]::GetFolderPath('Programs'),(Join-Path $env:APPDATA 'Microsoft/Internet Explorer/Quick Launch/User Pinned/TaskBar'))) {
    if (!(Test-Path -LiteralPath $root)) { continue }
    Get-ChildItem -LiteralPath $root -Filter 'Luxmc.lnk' -Recurse -ErrorAction SilentlyContinue | ForEach-Object {
        $shortcut = $shell.CreateShortcut($_.FullName)
        if ($shortcut.TargetPath -in @((Join-Path $installDirectory 'luxmc-repair.exe'),(Join-Path $installDirectory 'luxmc.exe'))) { $shortcuts[$_.FullName] = [IO.File]::ReadAllBytes($_.FullName) }
    }
}
$protocolPath = 'HKCU:/Software/Classes/luxmc/shell/open/command'
$previousProtocol = if (Test-Path -LiteralPath $protocolPath) { (Get-ItemProperty -LiteralPath $protocolPath).'(default)' } else { $null }
$oldHash = (Get-FileHash -LiteralPath (Join-Path $installDirectory 'luxmc.exe') -Algorithm SHA256).Hash.ToLowerInvariant()
if ($oldHash -ne 'b13bbf8087b92091035edf421ba74e3c584b2e67a1af38c3efa112a71845cdcb') { throw 'The installed program changed; review it before replacing files.' }
try {
    foreach ($name in @('luxmc.exe','uninstall.exe')) {
        Copy-Item -LiteralPath (Join-Path $payloadDirectory $name) -Destination (Join-Path $installDirectory $name) -Force
        if ((Get-FileHash -LiteralPath (Join-Path $payloadDirectory $name) -Algorithm SHA256).Hash -ne (Get-FileHash -LiteralPath (Join-Path $installDirectory $name) -Algorithm SHA256).Hash) { throw 'Installed file does not match the package.' }
    }
    foreach ($path in $shortcuts.Keys) {
        $shortcut = $shell.CreateShortcut($path)
        $shortcut.TargetPath = Join-Path $installDirectory 'luxmc.exe'
        $shortcut.WorkingDirectory = $installDirectory
        $shortcut.IconLocation = (Join-Path $installDirectory 'luxmc.exe') + ',0'
        $shortcut.Save()
        if ($shell.CreateShortcut($path).TargetPath -ne (Join-Path $installDirectory 'luxmc.exe')) { throw 'Shortcut migration failed.' }
    }
    if ($previousProtocol -and $previousProtocol.Contains($installDirectory,[StringComparison]::OrdinalIgnoreCase)) {
        Set-Item -LiteralPath $protocolPath -Value ('"' + (Join-Path $installDirectory 'luxmc.exe') + '" "%1"')
    }
    $removedHelper = [IO.Path]::GetFullPath((Join-Path $installDirectory 'luxmc-repair.exe'))
    if ($removedHelper -ne (Join-Path $installDirectory 'luxmc-repair.exe')) { throw 'Unexpected removal target.' }
    if (Test-Path -LiteralPath $removedHelper) { Remove-Item -LiteralPath $removedHelper }
    foreach ($file in $beforeHashes.Keys) {
        if ((Get-FileHash -LiteralPath $file -Algorithm SHA256).Hash -ne $beforeHashes[$file]) { throw 'Protected data changed during the update.' }
    }
} catch {
    foreach ($name in @('luxmc.exe','luxmc-repair.exe','uninstall.exe')) {
        $file = Join-Path $backupDirectory $name
        if (Test-Path -LiteralPath $file) { Copy-Item -LiteralPath $file -Destination (Join-Path $installDirectory $name) -Force }
    }
    foreach ($path in $shortcuts.Keys) { [IO.File]::WriteAllBytes($path,$shortcuts[$path]) }
    if ($previousProtocol) { Set-Item -LiteralPath $protocolPath -Value $previousProtocol }
    throw
}
$downloadsInstaller = Join-Path $env:USERPROFILE 'Downloads/Lux MC Launcher - Inicio Seguro.exe'
Copy-Item -LiteralPath $installer -Destination $downloadsInstaller -Force
$report = [ordered]@{
    Version = '3.9.1'
    Build = 'Inicio Seguro'
    InstalledExe = Join-Path $installDirectory 'luxmc.exe'
    InstalledSha256 = (Get-FileHash -LiteralPath (Join-Path $installDirectory 'luxmc.exe') -Algorithm SHA256).Hash.ToLowerInvariant()
    PreviousSha256 = $oldHash
    RecoveryHelperRemoved = !(Test-Path -LiteralPath (Join-Path $installDirectory 'luxmc-repair.exe'))
    DirectLauncherShortcuts = $shortcuts.Count
    UninstallerUpdated = $true
    ProtectedFilesVerified = $beforeHashes.Count
    DataPreserved = $true
    BackupDirectory = $backupDirectory
    Installer = $downloadsInstaller
    InstallerSha256 = $installerReceipt.sha256
    LauncherStarted = $false
    RuntimeTested = $false
    SecuritySettingsChanged = $false
}
$report | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $validationDirectory 'installed-update.json') -Encoding utf8
$report | ConvertTo-Json
