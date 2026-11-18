$ErrorActionPreference = 'Stop'
$toolDirectory = [IO.Path]::GetFullPath((Join-Path $env:LOCALAPPDATA 'Luxmc-build/tools/7zip-26.04'))
$package = Join-Path $toolDirectory '7z2604-x64.msi'
if ((Get-FileHash -LiteralPath $package -Algorithm SHA256).Hash.ToLowerInvariant() -ne '0b01334a418654293513449f61d6bfc99e5196ca371e3c3dc961eda57cd535c6') { throw 'Official archive tool digest mismatch.' }
$outputDirectory = Join-Path $toolDirectory 'files'
New-Item -ItemType Directory -Path $outputDirectory -Force | Out-Null
$reader = New-Object -ComObject WindowsInstaller.Installer
$database = $reader.OpenDatabase($package,0)
$view = $database.OpenView('SELECT `Name`, `Data` FROM `_Streams`')
$view.Execute()
while ($record = $view.Fetch()) {
    $name = $record.StringData(1)
    if (!$name.EndsWith('.cab',[StringComparison]::OrdinalIgnoreCase)) { continue }
    if ([IO.Path]::GetFileName($name) -ne $name) { throw 'Unexpected cabinet name.' }
    $cabinet = Join-Path $toolDirectory $name
    $stream = [IO.File]::Create($cabinet)
    try {
        while ($chunk = $record.ReadStream(2,65536,1)) {
            $bytes = [byte[]]::new($chunk.Length)
            for ($index=0; $index -lt $chunk.Length; $index++) { $bytes[$index] = [byte][char]$chunk[$index] }
            $stream.Write($bytes,0,$bytes.Length)
        }
    } finally { $stream.Dispose() }
    & (Join-Path $env:SystemRoot 'System32/expand.exe') '-F:*' $cabinet $outputDirectory | Out-Null
    if ($LASTEXITCODE -ne 0) { throw 'Could not read the archive cabinet.' }
}
$view.Close()
$view = $database.OpenView('SELECT `File`, `FileName` FROM `File`')
$view.Execute()
while ($record = $view.Fetch()) {
    $identifier = $record.StringData(1)
    $name = $record.StringData(2).Split('|')[-1]
    if ($name -notin @('7z.exe','7z.dll')) { continue }
    if ([IO.Path]::GetFileName($identifier) -ne $identifier) { throw 'Unexpected archive identifier.' }
    Copy-Item -LiteralPath (Join-Path $outputDirectory $identifier) -Destination (Join-Path $outputDirectory $name) -Force
}
$view.Close()
Get-Item -LiteralPath (Join-Path $outputDirectory '7z.exe') | Select-Object FullName,Length
