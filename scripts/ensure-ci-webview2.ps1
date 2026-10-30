$ErrorActionPreference = 'Stop'
if ($env:GITHUB_ACTIONS -ne 'true') { throw 'This runtime setup is restricted to an ephemeral CI runner.' }

function Get-WebView2Version {
    foreach ($taskRegistryPath in @('HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}', 'HKCU:\SOFTWARE\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}')) {
        $taskVersion = (Get-ItemProperty -LiteralPath $taskRegistryPath -Name pv -ErrorAction SilentlyContinue).pv
        if ($taskVersion -and $taskVersion -ne '0.0.0.0') { return $taskVersion }
    }
}

$taskInstalledVersion = Get-WebView2Version
if ($taskInstalledVersion) { Write-Output "WebView2 runtime available: $taskInstalledVersion"; exit 0 }
$taskBootstrapper = Join-Path $env:RUNNER_TEMP 'Luxmc-MicrosoftEdgeWebview2Setup.exe'
Invoke-WebRequest -Uri 'https://go.microsoft.com/fwlink/p/?LinkId=2124703' -OutFile $taskBootstrapper
$taskSignature = Get-AuthenticodeSignature -LiteralPath $taskBootstrapper
if ($taskSignature.Status -ne 'Valid' -or $taskSignature.SignerCertificate.Subject -notmatch 'O=Microsoft Corporation') { throw 'WebView2 bootstrapper does not have a valid Microsoft signature.' }
$taskInstallation = Start-Process -FilePath $taskBootstrapper -ArgumentList '/silent', '/install' -WindowStyle Hidden -Wait -PassThru
$taskInstalledVersion = Get-WebView2Version
if (!$taskInstalledVersion) { throw "WebView2 runtime setup failed: $($taskInstallation.ExitCode)" }
Write-Output "WebView2 runtime installed: $taskInstalledVersion"
