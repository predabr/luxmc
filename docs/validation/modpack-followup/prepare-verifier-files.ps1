$ErrorActionPreference = 'Stop'
$validationDirectory = 'C:/Users/preda/OneDrive/Documenti/Luxmc/docs/validation/modpack-followup'
$archivePath = Join-Path $validationDirectory 'verifier.zip'
$destination = [IO.Path]::GetFullPath((Join-Path $validationDirectory 'verifier'))
New-Item -ItemType Directory -Path $destination -Force | Out-Null
$zip = [IO.Compression.ZipFile]::OpenRead($archivePath)
try {
    foreach ($entry in $zip.Entries) {
        $target = [IO.Path]::GetFullPath((Join-Path $destination $entry.FullName))
        if (!$target.StartsWith($destination + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) { throw 'Caminho inválido no artefato.' }
        if ($entry.FullName.EndsWith('/')) { New-Item -ItemType Directory -Path $target -Force | Out-Null; continue }
        New-Item -ItemType Directory -Path (Split-Path -Parent $target) -Force | Out-Null
        [IO.Compression.ZipFileExtensions]::ExtractToFile($entry, $target, $true)
    }
} finally { $zip.Dispose() }
$receipt = Get-Content -LiteralPath (Join-Path $validationDirectory 'remote-verifier.json') -Raw | ConvertFrom-Json
$metadata = Get-Content -LiteralPath (Join-Path $destination 'verifier.json') -Raw | ConvertFrom-Json
if ($metadata.commit -ne $receipt.workflowCommit -or $metadata.mainSha256 -ne $receipt.mainSha256) { throw 'O verificador não corresponde à compilação esperada.' }
$helpers = @(Get-ChildItem -LiteralPath $destination -Recurse -File -Filter 'luxmc-repair.exe')
if ($helpers.Count -ne 1 -or (Get-FileHash -LiteralPath $helpers[0].FullName -Algorithm SHA256).Hash.ToLowerInvariant() -ne $metadata.helperSha256) { throw 'Hash do verificador inválido.' }
Copy-Item -LiteralPath $helpers[0].FullName -Destination 'C:/Users/preda/OneDrive/Documenti/Luxmc/src-tauri/binaries/luxmc-repair-x86_64-pc-windows-msvc.exe' -Force
Copy-Item -LiteralPath $helpers[0].FullName -Destination 'C:/Users/preda/AppData/Local/Luxmc-build/target/release/luxmc-repair.exe' -Force
Write-Output 'Verificador recebido e validado para a nova compilação.'
