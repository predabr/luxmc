param([Parameter(Mandatory=$true)][string]$Installer)
$ErrorActionPreference = 'Stop'
$projectRoot = 'C:\Users\preda\OneDrive\Documenti\Luxmc'
$installedRoot = Join-Path $env:LOCALAPPDATA 'Luxmc'
$binary = Join-Path $installedRoot 'luxmc.exe'
$compiled = Join-Path $env:LOCALAPPDATA 'Luxmc-build\target\release\luxmc.exe'
$installerPath = (Resolve-Path -LiteralPath $Installer).Path
if (-not $installerPath.StartsWith($projectRoot + '\release-windows\', [StringComparison]::OrdinalIgnoreCase)) { throw 'Instalador fora da pasta de saída local.' }
foreach ($process in Get-Process luxmc -ErrorAction SilentlyContinue) {
    if ($process.Path -eq $binary) { Stop-Process -Id $process.Id; $process.WaitForExit(10000) | Out-Null }
}
$protectedFiles = @(
    (Join-Path $env:APPDATA 'github\Luxmc\data\luxmc.db'),
    (Join-Path $env:APPDATA 'github\Luxmc\data\bedrock-provider.json'),
    (Join-Path $env:APPDATA 'github\Luxmc\data\private-lan.properties'),
    (Join-Path $env:APPDATA 'io.github.luxmc.Luxmc\auth.json'),
    (Join-Path $env:APPDATA 'io.github.luxmc.Luxmc\settings.json')
)
$before = @{}
foreach ($path in $protectedFiles) { if (Test-Path -LiteralPath $path) { $before[$path] = (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash } }
$instancesRoot = Join-Path $env:APPDATA 'github\Luxmc\data\instances'
$instanceFolders = @(Get-ChildItem -LiteralPath $instancesRoot -Directory | Select-Object -ExpandProperty Name | Sort-Object)
$install = Start-Process -FilePath $installerPath -ArgumentList @('/S','/UPDATE') -WindowStyle Hidden -Wait -PassThru
if ($install.ExitCode -ne 0) { throw "Instalador encerrou com código $($install.ExitCode)." }
foreach ($path in $before.Keys) {
    if (-not (Test-Path -LiteralPath $path) -or (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash -ne $before[$path]) { throw "Dados protegidos mudaram durante a instalação: $path" }
}
$afterFolders = @(Get-ChildItem -LiteralPath $instancesRoot -Directory | Select-Object -ExpandProperty Name | Sort-Object)
if (Compare-Object $instanceFolders $afterFolders) { throw 'As pastas de instâncias mudaram durante a atualização.' }
function Get-NormalizedHash([string]$Path) {
    $bytes = [IO.File]::ReadAllBytes($Path)
    $text = [Text.Encoding]::ASCII.GetString($bytes)
    $replacement = [Text.Encoding]::ASCII.GetBytes('PE_VAR_UNK')
    $offset = $text.IndexOf('PE_VAR_NSS', [StringComparison]::Ordinal)
    while ($offset -ge 0) {
        [Array]::Copy($replacement, 0, $bytes, $offset, $replacement.Length)
        $offset = $text.IndexOf('PE_VAR_NSS', $offset + $replacement.Length, [StringComparison]::Ordinal)
    }
    $sha = [Security.Cryptography.SHA256]::Create()
    try { return [BitConverter]::ToString($sha.ComputeHash($bytes)).Replace('-','').ToLowerInvariant() }
    finally { $sha.Dispose() }
}
if ((Get-NormalizedHash $binary) -ne (Get-NormalizedHash $compiled)) { throw 'O binário instalado não corresponde à compilação local.' }
$verification = (& (Join-Path $installedRoot 'luxmc-repair.exe') --verify-only | Out-String | ConvertFrom-Json)
if (-not $verification.healthy -or $verification.repaired -or $verification.rolledBack) { throw 'O verificador local não confirmou o binário novo.' }
$report = [ordered]@{ LocalOnly=$true; Version='3.6.0'; InstallerExitCode=$install.ExitCode; ProtectedFilesVerified=$before.Count; InstanceFoldersPreserved=$true; InstalledBinaryMatchesBuild=$true; RepairVerifierHealthy=$true; InstallerSha256=(Get-FileHash -LiteralPath $installerPath -Algorithm SHA256).Hash.ToLowerInvariant() }
$report | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $projectRoot 'docs\validation\v3.6\input-motion\installed.json') -Encoding utf8
$report | ConvertTo-Json
