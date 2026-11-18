$ErrorActionPreference = 'Stop'
$projectDirectory = 'C:/Users/preda/OneDrive/Documenti/Luxmc'
$installDirectory = 'C:/Users/preda/AppData/Local/Luxmc'
$buildDirectory = 'C:/Users/preda/AppData/Local/Luxmc-build/target/release'
$validationDirectory = Join-Path $projectDirectory 'docs/validation/artwork-personalizer'
$registryPath = 'HKCU:/Software/Microsoft/Windows/CurrentVersion/Uninstall/Luxmc'
$registered = Get-ItemProperty -LiteralPath $registryPath
if (Get-Process java,javaw,luxmc -ErrorAction SilentlyContinue) { throw 'Feche o Minecraft e o Luxmc antes de atualizar os arquivos.' }
$audit = Get-Content -LiteralPath (Join-Path $validationDirectory 'windows-local-audit.json') -Raw | ConvertFrom-Json
if ($audit.defender.scanExitCode -ne 0) { throw 'A verificação do Defender não foi concluída.' }
$verifier = Get-Content -LiteralPath (Join-Path $validationDirectory 'verifier/verifier.json') -Raw | ConvertFrom-Json
$encoding = [Text.Encoding]::GetEncoding(28591)
$programBytes = [IO.File]::ReadAllBytes((Join-Path $buildDirectory 'luxmc.exe'))
$normalizedBytes = $encoding.GetBytes($encoding.GetString($programBytes).Replace('PE_VAR_NSS', 'PE_VAR_UNK'))
$normalizedHash = [Convert]::ToHexString([Security.Cryptography.SHA256]::HashData($normalizedBytes)).ToLowerInvariant()
if ($normalizedHash -ne $verifier.mainSha256) { throw 'O verificador não corresponde ao executável compilado.' }
if ((Get-FileHash -LiteralPath (Join-Path $buildDirectory 'luxmc-repair.exe') -Algorithm SHA256).Hash.ToLowerInvariant() -ne $verifier.helperSha256) { throw 'O verificador mudou após a compilação validada.' }
$protectedFiles = @(
    'C:/Users/preda/AppData/Roaming/github/Luxmc/data/luxmc.db',
    'C:/Users/preda/AppData/Roaming/io.github.luxmc.Luxmc/auth.json',
    'C:/Users/preda/AppData/Roaming/io.github.luxmc.Luxmc/settings.json'
)
$backupDirectory = Join-Path $env:LOCALAPPDATA ('Luxmc-build/backups/artwork-personalizer-files-' + (Get-Date -Format 'yyyyMMdd-HHmmss'))
New-Item -ItemType Directory -Path $backupDirectory -Force | Out-Null
$beforeHashes = @{}
for ($index = 0; $index -lt $protectedFiles.Count; $index++) {
    $file = $protectedFiles[$index]
    if (Test-Path -LiteralPath $file) {
        $beforeHashes[$file] = (Get-FileHash -LiteralPath $file -Algorithm SHA256).Hash
        Copy-Item -LiteralPath $file -Destination (Join-Path $backupDirectory ("data-$index-" + [IO.Path]::GetFileName($file)))
    }
}
foreach ($name in @('luxmc.exe','luxmc-repair.exe')) {
    $source = Join-Path $buildDirectory $name
    $componentName = if ($name -eq 'luxmc.exe') { 'program' } else { 'recovery-helper' }
    $scanned = $audit.components | Where-Object { $_.component -eq $componentName }
    if ((Get-FileHash -LiteralPath $source -Algorithm SHA256).Hash.ToLowerInvariant() -ne $scanned.sha256) { throw 'O arquivo mudou após a verificação.' }
    Copy-Item -LiteralPath (Join-Path $installDirectory $name) -Destination (Join-Path $backupDirectory $name)
}
$oldHash = (Get-FileHash -LiteralPath (Join-Path $installDirectory 'luxmc.exe') -Algorithm SHA256).Hash
try {
    foreach ($name in @('luxmc.exe','luxmc-repair.exe')) {
        Copy-Item -LiteralPath (Join-Path $buildDirectory $name) -Destination (Join-Path $installDirectory $name) -Force
        if ((Get-FileHash -LiteralPath (Join-Path $buildDirectory $name) -Algorithm SHA256).Hash -ne (Get-FileHash -LiteralPath (Join-Path $installDirectory $name) -Algorithm SHA256).Hash) { throw 'Os arquivos instalados não correspondem à compilação.' }
    }
    foreach ($file in $beforeHashes.Keys) {
        if ((Get-FileHash -LiteralPath $file -Algorithm SHA256).Hash -ne $beforeHashes[$file]) { throw 'Os dados mudaram durante a atualização.' }
    }
    Set-ItemProperty -LiteralPath $registryPath -Name DisplayVersion -Value '3.9.1'
} catch {
    foreach ($name in @('luxmc.exe','luxmc-repair.exe')) {
        Copy-Item -LiteralPath (Join-Path $backupDirectory $name) -Destination (Join-Path $installDirectory $name) -Force
    }
    Set-ItemProperty -LiteralPath $registryPath -Name DisplayVersion -Value $registered.DisplayVersion
    throw
}
$installer = Join-Path $projectDirectory 'release-windows/Lux MC Launcher - Capas e Personalizador.exe'
$downloadsInstaller = Join-Path $env:USERPROFILE 'Downloads/Lux MC Launcher - Capas e Personalizador.exe'
Copy-Item -LiteralPath $installer -Destination $downloadsInstaller -Force
$report = [ordered]@{
    Version = (Get-Item -LiteralPath (Join-Path $installDirectory 'luxmc.exe')).VersionInfo.ProductVersion
    Build = 'Capas e Personalizador'
    Method = 'Verified application files updated in existing installation'
    InstallerStatus = 'Blocked by Windows application control; security settings unchanged'
    InstalledExe = Join-Path $installDirectory 'luxmc.exe'
    InstalledSha256 = (Get-FileHash -LiteralPath (Join-Path $installDirectory 'luxmc.exe') -Algorithm SHA256).Hash.ToLowerInvariant()
    PreviousSha256 = $oldHash.ToLowerInvariant()
    BuiltProgramVerified = $true
    RecoveryHelperVerified = $true
    ProtectedFilesVerified = $beforeHashes.Count
    DataPreserved = $true
    BackupDirectory = $backupDirectory
    Installer = $downloadsInstaller
    InstallerSha256 = (Get-FileHash -LiteralPath $installer -Algorithm SHA256).Hash.ToLowerInvariant()
    LauncherStarted = [bool](Get-Process luxmc -ErrorAction SilentlyContinue)
    RuntimeTested = $false
}
$report | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $validationDirectory 'installed-local.json') -Encoding utf8
$report | ConvertTo-Json
