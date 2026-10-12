param(
    [Parameter(Mandatory = $true)][string]$Installer,
    [Parameter(Mandatory = $true)][string]$Program,
    [string]$Uninstaller,
    [Parameter(Mandatory = $true)][string]$Output
)

$ErrorActionPreference = 'Stop'
$components = @()
$entries = @(@{ Name = 'installer'; Path = $Installer }, @{ Name = 'program'; Path = $Program })
if ($Uninstaller) { $entries += @{ Name = 'uninstaller'; Path = $Uninstaller } }
foreach ($entry in $entries) {
    $item = Get-Item -LiteralPath $entry.Path
    $signature = Get-AuthenticodeSignature -LiteralPath $item.FullName
    $components += [ordered]@{
        component = $entry.Name
        filename = $item.Name
        bytes = $item.Length
        sha256 = (Get-FileHash -LiteralPath $item.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
        authenticode = $signature.Status.ToString()
        publisher = if ($signature.SignerCertificate) { $signature.SignerCertificate.Subject } else { $null }
    }
}
$defender = Get-MpComputerStatus
$scanner = Join-Path $env:ProgramFiles 'Windows Defender/MpCmdRun.exe'
if (!$defender.AMServiceEnabled -or !$defender.AntivirusEnabled -or !(Test-Path -LiteralPath $scanner)) { throw 'Microsoft Defender is unavailable for release validation.' }
$scanOutput = @()
foreach ($entry in $entries) {
    $scanOutput += (& $scanner -Scan -ScanType 3 -File (Resolve-Path -LiteralPath $entry.Path).Path -DisableRemediation 2>&1 | Out-String)
    if ($LASTEXITCODE -ne 0) { throw "Microsoft Defender did not approve the scan (exit $LASTEXITCODE)." }
}
$report = [ordered]@{
    version = (Get-Content -LiteralPath 'package.json' -Raw | ConvertFrom-Json).version
    timestamp = [DateTime]::UtcNow.ToString('o')
    commit = $env:GITHUB_SHA
    run = if ($env:GITHUB_RUN_ID) { "https://github.com/$env:GITHUB_REPOSITORY/actions/runs/$env:GITHUB_RUN_ID" } else { $null }
    components = $components
    defender = [ordered]@{ signatureVersion = $defender.AntivirusSignatureVersion; scanExitCode = 0; output = $scanOutput }
    windowsTrustConfirmed = $false
    independentVendorApprovalConfirmed = $false
}
$directory = Split-Path -Parent $Output
if ($directory) { New-Item -ItemType Directory -Path $directory -Force | Out-Null }
$report | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $Output -Encoding utf8
Write-Output "Windows release audit saved to $Output. Authenticode and independent vendor approval are reported separately."
