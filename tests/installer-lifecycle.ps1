$ErrorActionPreference = 'Stop'
$projectRoot = [IO.Path]::GetFullPath((Get-Location).Path)
$testBase = [IO.Path]::GetFullPath((Join-Path $env:TEMP 'Luxmc-lifecycle'))
$testDirectory = [IO.Path]::GetFullPath((Join-Path $testBase ([guid]::NewGuid().ToString('N'))))
if (!$testDirectory.StartsWith($testBase + '\', [StringComparison]::OrdinalIgnoreCase) -or $testDirectory.StartsWith($projectRoot + '\', [StringComparison]::OrdinalIgnoreCase)) { throw 'Unsafe installation test directory.' }
$registration = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\Luxmc'
$ownedData = @((Join-Path $env:APPDATA 'github\Luxmc'),(Join-Path $env:APPDATA 'io.github.luxmc.Luxmc'),(Join-Path $env:LOCALAPPDATA 'github\Luxmc'),(Join-Path $env:LOCALAPPDATA 'io.github.luxmc.Luxmc'),(Join-Path $env:LOCALAPPDATA 'LuxmcRecovery'))
$canTestPurge = $env:GITHUB_ACTIONS -eq 'true' -and !@($ownedData | Where-Object { Test-Path -LiteralPath $_ }).Count
foreach ($root in @('HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall','HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall','HKLM:\Software\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall')) {
    $found = @(Get-ChildItem $root -ErrorAction SilentlyContinue | Get-ItemProperty | Where-Object DisplayName -Match '^Luxmc$|^Lux MC Launcher$')
    if ($found.Count) { throw 'An existing user installation prevents this isolated lifecycle test.' }
}
if (Get-Process luxmc -ErrorAction SilentlyContinue) { throw 'Close the launcher before the isolated installer test.' }
$protectedPaths = @('package.json','src/app.css','src-tauri/tauri.conf.json','src-tauri/src/core/launcher/mod.rs') | ForEach-Object { Join-Path $projectRoot $_ }
$dataRoot = Join-Path $env:APPDATA 'github\Luxmc\data'
foreach ($file in @('luxmc.db','settings.json')) { $protectedPaths += Join-Path $dataRoot $file }
foreach ($file in @('settings.json','auth.json')) { $protectedPaths += Join-Path (Join-Path $env:APPDATA 'io.github.luxmc.Luxmc') $file }
$protectedPaths += @(Get-ChildItem -LiteralPath (Join-Path $dataRoot 'instances') -Filter level.dat -File -Recurse -ErrorAction SilentlyContinue | ForEach-Object FullName)
$hashes = @{}
foreach ($path in $protectedPaths) { if (Test-Path -LiteralPath $path) { $hashes[$path] = (Get-FileHash -LiteralPath $path).Hash } }
$shortcutPaths = @((Join-Path ([Environment]::GetFolderPath('Desktop')) 'Luxmc.lnk'),(Join-Path ([Environment]::GetFolderPath('Programs')) 'Luxmc.lnk'))
$shortcutBackup = @{}
foreach ($path in $shortcutPaths) { if (Test-Path -LiteralPath $path) { $shortcutBackup[$path] = [IO.File]::ReadAllBytes($path) } }
$installer = if ($env:LUXMC_INSTALLER_PATH) { [IO.Path]::GetFullPath($env:LUXMC_INSTALLER_PATH) } else { Join-Path $projectRoot 'release-windows\Lux MC Launcher.exe' }
$releaseVersion = (Get-Content -LiteralPath (Join-Path $projectRoot 'package.json') -Raw | ConvertFrom-Json).version
$binary = Join-Path $testDirectory 'luxmc.exe'
$uninstaller = Join-Path $testDirectory 'uninstall.exe'
New-Item -ItemType Directory -Path $testDirectory -Force | Out-Null
try {
    $install = Start-Process -FilePath $installer -ArgumentList @('/S',"/D=$testDirectory") -WindowStyle Hidden -Wait -PassThru
    if ($install.ExitCode -ne 0 -or !(Test-Path -LiteralPath $binary)) { throw 'Isolated installation failed.' }
    $installed = Get-ItemProperty $registration
    if ($installed.DisplayVersion -ne $releaseVersion -or $installed.InstallLocation.Trim('"') -ne $testDirectory) { throw 'Unexpected installer registration.' }
    if ($installed.QuietUninstallString -notmatch '/S$') { throw 'Silent uninstall registration is missing.' }
    $reinstall = Start-Process -FilePath $installer -ArgumentList @('/S',"/D=$testDirectory") -WindowStyle Hidden -Wait -PassThru
    if ($reinstall.ExitCode -ne 0 -or !(Test-Path -LiteralPath $binary)) { throw 'Same-version reinstallation failed.' }
    $removal = Start-Process -FilePath $uninstaller -ArgumentList '/S' -WindowStyle Hidden -Wait -PassThru
    if ($removal.ExitCode -ne 0) { throw 'Isolated uninstallation failed.' }
    for ($attempt = 0; $attempt -lt 20 -and ((Test-Path -LiteralPath $binary) -or (Test-Path -LiteralPath $uninstaller)); $attempt++) { Start-Sleep -Milliseconds 500 }
    if ((Test-Path -LiteralPath $binary) -or (Test-Path -LiteralPath $uninstaller) -or (Test-Path $registration)) { throw 'Application or registration remained after uninstallation.' }
    foreach ($path in $hashes.Keys) { if (!(Test-Path -LiteralPath $path) -or (Get-FileHash -LiteralPath $path).Hash -ne $hashes[$path]) { throw "Protected file changed: $path" } }
    $purgeVerified = $false
    if ($canTestPurge) {
        $fixtureDocument = Join-Path ([Environment]::GetFolderPath('MyDocuments')) ('Luxmc-project-fixture-' + [guid]::NewGuid().ToString('N') + '.txt')
        [IO.File]::WriteAllText($fixtureDocument,'Project must remain')
        $fixtureHash = (Get-FileHash -LiteralPath $fixtureDocument).Hash
        $purgeInstall = Start-Process -FilePath $installer -ArgumentList @('/S',"/D=$testDirectory") -WindowStyle Hidden -Wait -PassThru
        if ($purgeInstall.ExitCode -ne 0) { throw 'Purge fixture installation failed.' }
        foreach ($path in $ownedData) { New-Item -ItemType Directory -Path (Join-Path $path 'fixture\instances\world') -Force | Out-Null; [IO.File]::WriteAllText((Join-Path $path 'fixture\instances\world\level.dat'),'Disposable test world') }
        $purgeRemoval = Start-Process -FilePath $uninstaller -ArgumentList @('/S','/PURGE') -WindowStyle Hidden -Wait -PassThru
        if ($purgeRemoval.ExitCode -ne 0) { throw 'Complete uninstallation failed.' }
        for ($attempt = 0; $attempt -lt 20 -and (Test-Path -LiteralPath $uninstaller); $attempt++) { Start-Sleep -Milliseconds 500 }
        if (@($ownedData | Where-Object { Test-Path -LiteralPath $_ }).Count -or (Test-Path -LiteralPath $binary) -or (Test-Path $registration)) { throw 'Files remained after complete uninstallation.' }
        if ((Get-FileHash -LiteralPath $fixtureDocument).Hash -ne $fixtureHash) { throw 'Documents fixture was modified.' }
        foreach ($path in $hashes.Keys) { if ((Get-FileHash -LiteralPath $path).Hash -ne $hashes[$path]) { throw "Project modified: $path" } }
        Remove-Item -LiteralPath $fixtureDocument
        $purgeVerified = $true
    }
    $receipt = [ordered]@{ Version=$releaseVersion; InstallExitCode=$install.ExitCode; SameVersionReinstallExitCode=$reinstall.ExitCode; UninstallExitCode=$removal.ExitCode; ApplicationRemoved=$true; RegistryRemoved=$true; ProtectedFiles=$hashes.Count; ProtectedDataPreserved=$true; CompleteCleanupVerified=$purgeVerified; InstallerSha256=(Get-FileHash -LiteralPath $installer).Hash }
    $receipt | ConvertTo-Json | Set-Content -Encoding utf8 'docs/validation/general-installer-lifecycle-2026-10-06.json'
    $receipt | ConvertTo-Json
} finally {
    foreach ($path in $shortcutBackup.Keys) { [IO.File]::WriteAllBytes($path,$shortcutBackup[$path]) }
}
