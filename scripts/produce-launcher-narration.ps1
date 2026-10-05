param(
    [Parameter(Mandatory = $true)][string]$Manifest,
    [string]$Voice = 'Microsoft Daniel'
)
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Speech
$taskNarration = Get-Content -LiteralPath $Manifest -Raw -Encoding UTF8 | ConvertFrom-Json
$taskSpeaker = New-Object System.Speech.Synthesis.SpeechSynthesizer
try {
    $taskVoiceNames = @($taskSpeaker.GetInstalledVoices() | ForEach-Object { $_.VoiceInfo.Name })
    if ($taskVoiceNames -notcontains $Voice) {
        $Voice = @($taskSpeaker.GetInstalledVoices() | Where-Object { $_.VoiceInfo.Culture.Name -like 'pt-*' } | ForEach-Object { $_.VoiceInfo.Name })[0]
    }
    if (-not $Voice) { throw 'No Portuguese speech voice is installed' }
    $taskSpeaker.SelectVoice($Voice)
    $taskSpeaker.Rate = 0
    $taskSpeaker.Volume = 100
    foreach ($taskItem in $taskNarration) {
        $taskOutput = [IO.Path]::GetFullPath([string]$taskItem.path)
        $taskSpeaker.SetOutputToWaveFile($taskOutput)
        $taskSpeaker.Speak([string]$taskItem.text)
        $taskSpeaker.SetOutputToNull()
    }
    Write-Output "Narration complete: $($taskNarration.Count) clips; voice=$Voice"
} finally {
    $taskSpeaker.Dispose()
}
