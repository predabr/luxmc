$ErrorActionPreference = 'Stop'
$version = (Get-Content -LiteralPath 'package.json' -Raw | ConvertFrom-Json).version
$project = [IO.Path]::GetFullPath((Get-Location).Path)
$testRoot = [IO.Path]::GetFullPath((Join-Path $env:LOCALAPPDATA 'Luxmc-build\tests'))
$target = [IO.Path]::GetFullPath((Join-Path $testRoot ('upgrade-' + $version)))
if (!$target.StartsWith($testRoot + '\', [StringComparison]::OrdinalIgnoreCase) -or $target.StartsWith($project + '\', [StringComparison]::OrdinalIgnoreCase)) { throw 'Unsafe upgrade directory.' }
$registration = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\Luxmc'
if (Test-Path $registration) { throw 'A user installation is present.' }
if (Get-Process luxmc -ErrorAction SilentlyContinue) { throw 'Launcher running.' }
if ((Test-Path -LiteralPath $target) -and @(Get-ChildItem -LiteralPath $target -Force).Count -gt 0) { throw 'Upgrade directory is not empty.' }
New-Item -ItemType Directory -Path $target -Force | Out-Null
$previous = Join-Path $testRoot 'previous-3.0.1.exe'
$installer = Join-Path $project 'release-windows\Lux MC Launcher.exe'
$protected = @((Join-Path $project 'Luxmc.exe'), (Join-Path $env:APPDATA 'github\Luxmc\data\luxmc.db'), (Join-Path $env:APPDATA 'io.github.luxmc.Luxmc\auth.json'), (Join-Path $env:APPDATA 'io.github.luxmc.Luxmc\settings.json'))
$hashes = @{}
foreach ($file in $protected) { if (Test-Path -LiteralPath $file) { $hashes[$file] = (Get-FileHash -LiteralPath $file).Hash } }
$old = Start-Process -FilePath $previous -ArgumentList @('/S', "/D=$target") -WindowStyle Hidden -Wait -PassThru
if ($old.ExitCode -ne 0) { throw 'Previous installation failed.' }
$oldVersion = (Get-ItemProperty $registration).DisplayVersion
if ($oldVersion -ne '3.0.1') { throw 'Unexpected previous version.' }
$marker = Join-Path $target 'upgrade-preserved.txt'
'Preserve existing installation content.' | Set-Content -LiteralPath $marker
$updated = Start-Process -FilePath $installer -ArgumentList @('/P', '/UPDATE', "/D=$target") -WindowStyle Hidden -Wait -PassThru
if ($updated.ExitCode -ne 0) { throw 'Passive update failed.' }
$registered = Get-ItemProperty $registration
if ($registered.DisplayVersion -ne $version -or $registered.InstallLocation.Trim('"') -ne $target) { throw 'Update did not replace the registered version.' }
if (!(Test-Path -LiteralPath $marker)) { throw 'Update removed existing installation content.' }
$installedHash = (Get-FileHash -LiteralPath (Join-Path $target 'luxmc.exe')).Hash
$expectedHash = (Get-FileHash -LiteralPath (Join-Path $env:LOCALAPPDATA 'Luxmc-build\target\release\luxmc.exe')).Hash
if ($installedHash -ne $expectedHash) { throw 'Updated executable differs from the production binary.' }
foreach ($file in $hashes.Keys) { if ((Get-FileHash -LiteralPath $file).Hash -ne $hashes[$file]) { throw "Protected data changed: $file" } }
$removal = Start-Process -FilePath (Join-Path $target 'uninstall.exe') -ArgumentList '/S' -WindowStyle Hidden -Wait -PassThru
Start-Sleep -Seconds 2
if ($removal.ExitCode -ne 0 -or (Test-Path -LiteralPath (Join-Path $target 'luxmc.exe')) -or (Test-Path $registration)) { throw 'Updated application could not be uninstalled.' }
Remove-Item -LiteralPath $marker
$receipt = [ordered]@{Version=$version;PreviousVersion=$oldVersion;UpdateExitCode=$updated.ExitCode;UpdatedBinaryVerified=$true;ExistingContentPreserved=$true;ProtectedDataPreserved=$true;UpdatedAppUninstalled=$true}
$receipt | ConvertTo-Json | Set-Content (Join-Path $testRoot ('upgrade-' + $version + '.json'))
$receipt | ConvertTo-Json
