$ErrorActionPreference = 'Stop'
$projectPath = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..')).TrimEnd('\')
$installPath = Join-Path $env:LOCALAPPDATA 'Luxmc'
$targets = @(
    (Join-Path $env:APPDATA 'github\Luxmc'),
    (Join-Path $env:LOCALAPPDATA 'github\Luxmc'),
    (Join-Path $env:APPDATA 'io.github.luxmc.Luxmc'),
    (Join-Path $env:LOCALAPPDATA 'io.github.luxmc.Luxmc'),
    (Join-Path $env:LOCALAPPDATA 'LuxmcRecovery'),
    $installPath
)
foreach ($target in $targets) {
    $absolute = [IO.Path]::GetFullPath($target).TrimEnd('\')
    if ($absolute.Equals($projectPath, [StringComparison]::OrdinalIgnoreCase) -or $absolute.StartsWith($projectPath + '\', [StringComparison]::OrdinalIgnoreCase) -or $projectPath.StartsWith($absolute + '\', [StringComparison]::OrdinalIgnoreCase)) { throw 'A limpeza atingiria o projeto protegido.' }
    if (!(($absolute.StartsWith($env:APPDATA + '\', [StringComparison]::OrdinalIgnoreCase)) -or ($absolute.StartsWith($env:LOCALAPPDATA + '\', [StringComparison]::OrdinalIgnoreCase)))) { throw 'Pasta fora dos dados do aplicativo.' }
    if (Test-Path -LiteralPath $absolute) {
        $item = Get-Item -LiteralPath $absolute -Force
        if ($item.Attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'Raiz de limpeza redirecionada: requer inspeção.' }
    }
}
$mainPath = Join-Path $installPath 'luxmc.exe'
Get-Process luxmc -ErrorAction SilentlyContinue | Where-Object { $_.Path -and $_.Path.Equals($mainPath, [StringComparison]::OrdinalIgnoreCase) } | ForEach-Object { Stop-Process -Id $_.Id }
$uninstaller = Join-Path $installPath 'uninstall.exe'
if (Test-Path -LiteralPath $uninstaller) {
    $process = Start-Process -FilePath $uninstaller -ArgumentList '/S' -WindowStyle Hidden -Wait -PassThru
    if ($process.ExitCode -ne 0) { throw "Desinstalador encerrou com código $($process.ExitCode)." }
}
$removed = @()
foreach ($target in $targets) {
    if (Test-Path -LiteralPath $target) {
        $links = @(Get-ChildItem -LiteralPath $target -Recurse -Force -Attributes ReparsePoint)
        foreach ($link in $links) {
            if ($link.PSIsContainer) { [IO.Directory]::Delete($link.FullName) }
            else { Remove-Item -LiteralPath $link.FullName -Force }
        }
        Remove-Item -LiteralPath $target -Recurse -Force
    }
    if (Test-Path -LiteralPath $target) { throw "Pasta não foi removida: $target" }
    $removed += $target
}
if (!(Test-Path -LiteralPath (Join-Path $projectPath 'package.json'))) { throw 'Verificação do projeto falhou.' }
[pscustomobject]@{ RemovedRoots=$removed; ProjectPreserved=$projectPath; Completed=$true } | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $projectPath 'docs\validation\clean-installed-luxmc.json') -Encoding utf8
'Launcher antigo e dados removidos. Projeto preservado.'
