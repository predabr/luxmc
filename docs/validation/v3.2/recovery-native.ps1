$ErrorActionPreference='Stop'
$taskDirectory=Join-Path $env:TEMP ('luxmc-recovery-3.2-'+[guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $taskDirectory | Out-Null
$taskMain=Join-Path $taskDirectory 'luxmc.exe'
$taskHelper=Join-Path $taskDirectory 'luxmc-repair.exe'
Copy-Item -LiteralPath 'C:/Users/preda/AppData/Local/Luxmc-build/target/release/luxmc.exe' -Destination $taskMain
Copy-Item -LiteralPath 'C:/Users/preda/AppData/Local/Luxmc-build/target/release/luxmc-repair.exe' -Destination $taskHelper
$taskInitialHash=(Get-FileHash -LiteralPath $taskMain).Hash
function Invoke-RecoveryProbe([string]$mode,[string]$label){
 $taskOutput=Join-Path $taskDirectory ($label+'.json');$taskError=Join-Path $taskDirectory ($label+'.err')
 $taskProcess=Start-Process -FilePath $taskHelper -ArgumentList $mode -WindowStyle Hidden -Wait -PassThru -RedirectStandardOutput $taskOutput -RedirectStandardError $taskError
 if($taskProcess.ExitCode -ne 0){throw (Get-Content -LiteralPath $taskError -Raw)}
 return Get-Content -LiteralPath $taskOutput -Raw | ConvertFrom-Json
}
$taskInitialized=Invoke-RecoveryProbe '--initialize' 'initialize'
if($taskInitialized.version -ne '3.2.0' -or !$taskInitialized.healthy){throw 'Incorrect recovery initialization'}
$taskBytes=[IO.File]::ReadAllBytes($taskMain);$taskBytes[100]=($taskBytes[100] -bxor 1);[IO.File]::WriteAllBytes($taskMain,$taskBytes)
$taskBroken=Invoke-RecoveryProbe '--verify-only' 'broken'
if($taskBroken.healthy){throw 'Corruption was not detected'}
$taskRestored=Invoke-RecoveryProbe '--repair-only' 'restored'
if(!$taskRestored.healthy -or !$taskRestored.repaired -or (Get-FileHash -LiteralPath $taskMain).Hash -ne $taskInitialHash){throw 'Repair did not restore the original bytes'}
$taskResult=[ordered]@{Version='3.2.0';CorruptionDetected=$true;OriginalBytesRestored=$true;UserInstallationUnchanged=$true;IsolatedDirectory=$taskDirectory}
$taskResult | ConvertTo-Json | Set-Content -Encoding utf8 docs/validation/v3.2/recovery-native.json
$taskResult | ConvertTo-Json
