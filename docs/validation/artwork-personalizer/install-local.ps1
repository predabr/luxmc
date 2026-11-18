param(
    [string]$Installer = 'C:/Users/preda/OneDrive/Documenti/Luxmc/release-windows/Lux MC Launcher - Capas e Personalizador.exe'
)

$ErrorActionPreference = 'Stop'
$installDirectory = 'C:/Users/preda/AppData/Local/Luxmc'
$installedBinary = Join-Path $installDirectory 'luxmc.exe'
$builtBinary = 'C:/Users/preda/AppData/Local/Luxmc-build/target/release/luxmc.exe'
$builtHelper = 'C:/Users/preda/AppData/Local/Luxmc-build/target/release/luxmc-repair.exe'
$projectDirectory = 'C:/Users/preda/OneDrive/Documenti/Luxmc'
if (!(Test-Path -LiteralPath $Installer) -or !(Test-Path -LiteralPath $builtBinary)) { throw 'A nova compilação ainda não está pronta.' }
if (Get-Process java,javaw -ErrorAction SilentlyContinue) { throw 'Java está aberto. A atualização foi adiada para preservar o jogo.' }
foreach ($launcher in @(Get-Process luxmc -ErrorAction SilentlyContinue | Where-Object { $_.Path -eq $installedBinary })) {
    [void]$launcher.CloseMainWindow()
    if (!$launcher.WaitForExit(5000)) { Stop-Process -Id $launcher.Id; $launcher.WaitForExit() }
}

function Get-NormalizedProgramHash([string]$Path) {
    $encoding = [Text.Encoding]::GetEncoding(28591)
    $bytes = [IO.File]::ReadAllBytes($Path)
    $text = $encoding.GetString($bytes).Replace('PE_VAR_NSS', 'PE_VAR_UNK')
    return [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($encoding.GetBytes($text))).ToLowerInvariant()
}

$protectedFiles = @(
    'C:/Users/preda/AppData/Roaming/github/Luxmc/data/luxmc.db',
    'C:/Users/preda/AppData/Roaming/io.github.luxmc.Luxmc/auth.json',
    'C:/Users/preda/AppData/Roaming/io.github.luxmc.Luxmc/settings.json'
)
$backupDirectory = Join-Path $env:LOCALAPPDATA ('Luxmc-build/backups/artwork-personalizer-' + (Get-Date -Format 'yyyyMMdd-HHmmss'))
New-Item -ItemType Directory -Path $backupDirectory -Force | Out-Null
$beforeHashes = @{}
for ($index = 0; $index -lt $protectedFiles.Count; $index++) {
    $dataFile = $protectedFiles[$index]
    if (Test-Path -LiteralPath $dataFile) {
        $beforeHashes[$dataFile] = (Get-FileHash -LiteralPath $dataFile -Algorithm SHA256).Hash
        Copy-Item -LiteralPath $dataFile -Destination (Join-Path $backupDirectory ("data-$index-" + [IO.Path]::GetFileName($dataFile)))
    }
}
foreach ($name in @('luxmc.exe','luxmc-repair.exe','uninstall.exe')) {
    $file = Join-Path $installDirectory $name
    if (Test-Path -LiteralPath $file) { Copy-Item -LiteralPath $file -Destination (Join-Path $backupDirectory $name) }
}
$oldHash = (Get-FileHash -LiteralPath $installedBinary -Algorithm SHA256).Hash
$expected = Get-NormalizedProgramHash $builtBinary
if ((Get-NormalizedProgramHash $installedBinary) -eq $expected) { throw 'O launcher já corresponde a essa compilação.' }

$installation = Start-Process -FilePath $Installer -ArgumentList @('/S', "/D=$installDirectory") -WindowStyle Hidden -Wait -PassThru
if ($installation.ExitCode -ne 0) { throw "A instalação falhou com código $($installation.ExitCode)." }
if ((Get-NormalizedProgramHash $installedBinary) -ne $expected) { throw 'O executável instalado não corresponde à nova compilação.' }
if ((Get-FileHash -LiteralPath (Join-Path $installDirectory 'luxmc-repair.exe')).Hash -ne (Get-FileHash -LiteralPath $builtHelper).Hash) { throw 'O verificador instalado não corresponde à nova compilação.' }
foreach ($dataFile in $beforeHashes.Keys) {
    if ((Get-FileHash -LiteralPath $dataFile -Algorithm SHA256).Hash -ne $beforeHashes[$dataFile]) { throw 'Um arquivo de dados mudou durante a instalação. Confira o backup antes de continuar.' }
}
$registered = Get-ItemProperty 'HKCU:/Software/Microsoft/Windows/CurrentVersion/Uninstall/Luxmc'
$newHash = (Get-FileHash -LiteralPath $installedBinary -Algorithm SHA256).Hash
if ($newHash -eq $oldHash -or $registered.DisplayVersion -ne '3.9.1') { throw 'A atualização instalada não pôde ser confirmada.' }
$downloadsInstaller = Join-Path $env:USERPROFILE 'Downloads/Lux MC Launcher - Capas e Personalizador.exe'
Copy-Item -LiteralPath $Installer -Destination $downloadsInstaller -Force
$report = [ordered]@{
    Version = $registered.DisplayVersion
    Build = 'Capas e Personalizador'
    InstallExitCode = $installation.ExitCode
    InstalledExe = $installedBinary
    InstalledSha256 = $newHash.ToLowerInvariant()
    BuiltProgramVerified = $true
    RecoveryHelperVerified = $true
    ProtectedFilesVerified = $beforeHashes.Count
    DataPreserved = $true
    BackupDirectory = $backupDirectory
    Installer = $downloadsInstaller
    InstallerSha256 = (Get-FileHash -LiteralPath $Installer -Algorithm SHA256).Hash.ToLowerInvariant()
    LauncherStarted = [bool](Get-Process luxmc -ErrorAction SilentlyContinue)
}
$report | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $projectDirectory 'docs/validation/artwork-personalizer/installed-local.json') -Encoding utf8
$report | ConvertTo-Json
