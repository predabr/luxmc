$ErrorActionPreference = 'Stop'
$sourceInstaller = Join-Path (Get-Location).Path 'release-windows\Lux MC Launcher.exe'
$installDirectory = Join-Path $env:LOCALAPPDATA 'Luxmc'
$installedBinary = Join-Path $installDirectory 'luxmc.exe'
if (-not (Test-Path -LiteralPath $sourceInstaller)) { throw 'Instalador não encontrado.' }
if (Get-Process java,javaw -ErrorAction SilentlyContinue) { throw 'Java está aberto; atualização adiada para preservar o jogo.' }
$protectedFiles = @(
    (Join-Path $env:APPDATA 'github\Luxmc\data\luxmc.db'),
    (Join-Path $env:APPDATA 'io.github.luxmc.Luxmc\auth.json'),
    (Join-Path $env:APPDATA 'io.github.luxmc.Luxmc\settings.json')
)
Get-Process luxmc -ErrorAction SilentlyContinue | Where-Object { $_.Path -eq $installedBinary } | Stop-Process
Start-Sleep -Seconds 2
$beforeHashes = @{}
foreach ($dataFile in $protectedFiles) {
    if (Test-Path -LiteralPath $dataFile) { $beforeHashes[$dataFile] = (Get-FileHash -LiteralPath $dataFile -Algorithm SHA256).Hash }
}
$oldHash = if (Test-Path -LiteralPath $installedBinary) { (Get-FileHash -LiteralPath $installedBinary -Algorithm SHA256).Hash } else { $null }
$installation = Start-Process -FilePath $sourceInstaller -ArgumentList @('/S', "/D=$installDirectory") -WindowStyle Hidden -Wait -PassThru
if ($installation.ExitCode -ne 0) { throw "Instalação falhou: $($installation.ExitCode)" }
foreach ($dataFile in $beforeHashes.Keys) {
    if ((Get-FileHash -LiteralPath $dataFile -Algorithm SHA256).Hash -ne $beforeHashes[$dataFile]) { throw "Dados alterados durante a instalação: $dataFile" }
}
$installedHash = (Get-FileHash -LiteralPath $installedBinary -Algorithm SHA256).Hash
if ($installedHash -eq $oldHash) { throw 'O executável instalado não foi substituído.' }
$metadata = [ordered]@{
    Version = '3.0.2'
    InstallerPath = $sourceInstaller
    Size = (Get-Item -LiteralPath $sourceInstaller).Length
    Sha256 = (Get-FileHash -LiteralPath $sourceInstaller -Algorithm SHA256).Hash
    InstallExitCode = $installation.ExitCode
    InstalledExe = $installedBinary
    InstalledSha256 = $installedHash
    ProtectedFilesVerified = $beforeHashes.Count
    DataPreservedDuringInstallation = $true
}
$metadata | ConvertTo-Json | Set-Content -LiteralPath 'docs/validation/final-polish-install.json' -Encoding utf8
$metadata | ConvertTo-Json
