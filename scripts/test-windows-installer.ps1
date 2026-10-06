$ErrorActionPreference = 'Stop'
$expectedVersion = (Get-Content -LiteralPath 'package.json' -Raw | ConvertFrom-Json).version
$projectRoot = [IO.Path]::GetFullPath((Get-Location).Path)
$testRoot = [IO.Path]::GetFullPath((Join-Path $env:LOCALAPPDATA 'Luxmc-build\tests'))
$sandbox = [IO.Path]::GetFullPath((Join-Path $testRoot ('uninstall-' + $expectedVersion)))
if (!$sandbox.StartsWith($testRoot + '\', [StringComparison]::OrdinalIgnoreCase) -or $sandbox.StartsWith($projectRoot + '\', [StringComparison]::OrdinalIgnoreCase)) { throw 'Unsafe uninstaller test directory.' }
$registration = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\Luxmc'
if (Test-Path $registration) { throw 'A user installation is present. The isolated test will not replace it.' }
if (Get-Process luxmc -ErrorAction SilentlyContinue) { throw 'Launcher running: isolated test postponed.' }
if ((Test-Path -LiteralPath $sandbox) -and @(Get-ChildItem -LiteralPath $sandbox -Force).Count -gt 0) { throw 'Uninstaller test directory is not empty.' }
New-Item -ItemType Directory -Path $sandbox -Force | Out-Null
$protectedPaths = @((Join-Path $projectRoot 'Luxmc.exe'), (Join-Path $env:APPDATA 'github\Luxmc\data\luxmc.db'), (Join-Path $env:APPDATA 'io.github.luxmc.Luxmc\auth.json'), (Join-Path $env:APPDATA 'io.github.luxmc.Luxmc\settings.json'))
$hashes = @{}
foreach ($path in $protectedPaths) { if (Test-Path -LiteralPath $path) { $hashes[$path] = (Get-FileHash -LiteralPath $path).Hash } }
$installer = Join-Path $projectRoot 'release-windows\Lux MC Launcher.exe'
$installation = Start-Process -FilePath $installer -ArgumentList @('/S', "/D=$sandbox") -WindowStyle Hidden -Wait -PassThru
if ($installation.ExitCode -ne 0) { throw "Isolated installation failed: $($installation.ExitCode)" }
$installed = Get-ItemProperty $registration
if ($installed.DisplayVersion -ne $expectedVersion -or $installed.InstallLocation.Trim('"') -ne $sandbox) { throw 'Unexpected installer registration.' }
if ($installed.QuietUninstallString -notmatch '/S$') { throw 'Silent uninstaller registration missing.' }
$binary = Join-Path $sandbox 'luxmc.exe'
if (!(Test-Path -LiteralPath $binary)) { throw 'Isolated launcher missing.' }
$uninstaller = Join-Path $sandbox 'uninstall.exe'
$removal = Start-Process -FilePath $uninstaller -ArgumentList '/S' -WindowStyle Hidden -Wait -PassThru
if ($removal.ExitCode -ne 0) { throw "Isolated uninstall failed: $($removal.ExitCode)" }
for ($attempt = 0; $attempt -lt 10 -and ((Test-Path -LiteralPath $binary) -or (Test-Path -LiteralPath $uninstaller)); $attempt++) { Start-Sleep -Milliseconds 500 }
if ((Test-Path -LiteralPath $binary) -or (Test-Path -LiteralPath $uninstaller) -or (Test-Path $registration)) { throw 'Uninstaller left the application or registry behind.' }
foreach ($path in $hashes.Keys) { if ((Get-FileHash -LiteralPath $path).Hash -ne $hashes[$path]) { throw "Protected data changed: $path" } }
$receipt = [ordered]@{Version=$expectedVersion;InstallExitCode=$installation.ExitCode;UninstallExitCode=$removal.ExitCode;ApplicationRemoved=$true;RegistryRemoved=$true;ProtectedDataPreserved=$true;UserAppStillUninstalled=(!(Test-Path -LiteralPath (Join-Path $env:LOCALAPPDATA 'Luxmc\luxmc.exe')))}
$receipt | ConvertTo-Json | Set-Content (Join-Path $testRoot ('installer-' + $expectedVersion + '.json'))
$receipt | ConvertTo-Json
