$ErrorActionPreference = 'Stop'
if ($env:GITHUB_ACTIONS -ne 'true') { throw 'UI capture is restricted to an ephemeral CI runner.' }
$taskReportDirectory = Join-Path $env:RUNNER_TEMP 'luxmc-native-ui'
New-Item -ItemType Directory -Path $taskReportDirectory -Force | Out-Null
Get-CimInstance Win32_Process | Where-Object { $_.Name -in @('luxmc.exe', 'msedgewebview2.exe', 'WerFault.exe') } | Select-Object ProcessId, ParentProcessId, Name, CommandLine | ConvertTo-Json -Depth 3 | Tee-Object -FilePath (Join-Path $taskReportDirectory 'processes.json')
Get-Process luxmc,msedgewebview2 -ErrorAction SilentlyContinue | Select-Object Id, MainWindowTitle, Responding | ConvertTo-Json | Tee-Object -FilePath (Join-Path $taskReportDirectory 'windows.json')
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing
$taskBounds = [System.Windows.Forms.SystemInformation]::VirtualScreen
$taskBitmap = New-Object System.Drawing.Bitmap $taskBounds.Width, $taskBounds.Height
$taskGraphics = [System.Drawing.Graphics]::FromImage($taskBitmap)
try {
    $taskGraphics.CopyFromScreen($taskBounds.Left, $taskBounds.Top, 0, 0, $taskBitmap.Size)
    $taskBitmap.Save((Join-Path $taskReportDirectory 'desktop.png'), [System.Drawing.Imaging.ImageFormat]::Png)
} finally { $taskGraphics.Dispose(); $taskBitmap.Dispose() }
