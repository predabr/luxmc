$ErrorActionPreference = 'Stop'
$projectRoot = (Get-Location).Path
$sourceInstaller = Join-Path $projectRoot 'release-windows\Lux MC Launcher.exe'
$destinationInstaller = Join-Path $env:USERPROFILE 'Downloads\Lux MC Launcher.exe'
$installedBinary = Join-Path $env:LOCALAPPDATA 'Luxmc\luxmc.exe'
if (Get-Process java,javaw -ErrorAction SilentlyContinue) { throw 'Minecraft/Java aberto: instalação adiada para preservar o jogo.' }
$protectedFiles = @(
    (Join-Path $env:APPDATA 'github\Luxmc\data\luxmc.db'),
    (Join-Path $env:APPDATA 'io.github.luxmc.Luxmc\auth.json'),
    (Join-Path $env:APPDATA 'io.github.luxmc.Luxmc\settings.json')
)
Get-Process luxmc -ErrorAction SilentlyContinue | Where-Object { $_.Path -eq $installedBinary } | Stop-Process
Start-Sleep -Seconds 2
$beforeHashes = @{}
foreach ($dataFile in $protectedFiles) { $beforeHashes[$dataFile] = (Get-FileHash -LiteralPath $dataFile -Algorithm SHA256).Hash }
Copy-Item -LiteralPath $sourceInstaller -Destination $destinationInstaller -Force
$installation = Start-Process -FilePath $destinationInstaller -ArgumentList '/S' -WindowStyle Hidden -Wait -PassThru
if ($installation.ExitCode -ne 0) { throw "Instalação falhou: $($installation.ExitCode)" }
foreach ($dataFile in $protectedFiles) {
    if ((Get-FileHash -LiteralPath $dataFile -Algorithm SHA256).Hash -ne $beforeHashes[$dataFile]) { throw "Dados alterados na instalação: $dataFile" }
}
Copy-Item -LiteralPath $installedBinary -Destination (Join-Path $projectRoot 'Luxmc.exe') -Force
$metadata = [ordered]@{
    InstallerPath = $destinationInstaller
    Size = (Get-Item -LiteralPath $destinationInstaller).Length
    Sha256 = (Get-FileHash -LiteralPath $destinationInstaller -Algorithm SHA256).Hash
    InstallExitCode = $installation.ExitCode
    InstalledExe = $installedBinary
    DataPreserved = $true
}
$metadata | ConvertTo-Json | Set-Content -LiteralPath 'docs/validation/wallpaper-transparency-delivery.json' -Encoding utf8
$metadata | ConvertTo-Json


