# Installs a packaged mhc for Dori on a Windows worker: the zip given as -Archive goes to %USERPROFILE%\.maho\dist\mhc,
# mhc-dori.cmd beside this script becomes %USERPROFILE%\.local\bin\mhc.cmd, the shadow home is prepared, the
# persistent launch spec is written, and the scheduled task \dori-mhc-host (at logon, interactive token, like
# \dori-omo-host) is registered and run once. Run it again to upgrade.
param([Parameter(Mandatory = $true)][string]$Archive)
$ErrorActionPreference = 'Stop'
$real = $env:USERPROFILE
$maho = Join-Path $real '.maho'
$dist = Join-Path $maho 'dist'
$bin = Join-Path $real '.local\bin'
New-Item -ItemType Directory -Force -Path $dist, $bin | Out-Null
$stage = Join-Path $dist ('.stage-' + [guid]::NewGuid().ToString('N'))
Expand-Archive -LiteralPath $Archive -DestinationPath $stage
$top = Get-ChildItem -LiteralPath $stage -Directory | Select-Object -First 1
if (-not (Test-Path -LiteralPath (Join-Path $top.FullName 'mhc.exe'))) { throw "$Archive has no mhc.exe" }
$current = Join-Path $dist 'mhc'
$old = Join-Path $dist 'mhc.old'
if (Test-Path -LiteralPath $old) { Remove-Item -LiteralPath $old -Recurse -Force }
if (Test-Path -LiteralPath $current) { Move-Item -LiteralPath $current -Destination $old }
Move-Item -LiteralPath $top.FullName -Destination $current
Remove-Item -LiteralPath $stage -Recurse -Force
Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'mhc-dori.cmd') -Destination (Join-Path $bin 'mhc.cmd') -Force
& (Join-Path $PSScriptRoot 'mhc-dori-setup.ps1')
$spec = Join-Path $maho 'dori-mhc-launch-spec.json'
Set-Content -LiteralPath $spec -Encoding ascii -Value '{"spec_version": 1, "core": {"session_runtime": "in-process", "multi_session": true, "extensions": []}, "tunables": {"idleExitMs": 900000, "coldStart": "persistent"}}'
$mhc = Join-Path $bin 'mhc.cmd'
& $mhc --version
$action = New-ScheduledTaskAction -Execute 'cmd.exe' -Argument "/c `"$mhc`" host ensure --launch-spec `"$spec`" --json"
$trigger = New-ScheduledTaskTrigger -AtLogOn -User "$env:USERDOMAIN\$env:USERNAME"
$principal = New-ScheduledTaskPrincipal -UserId "$env:USERDOMAIN\$env:USERNAME" -LogonType Interactive
$settings = New-ScheduledTaskSettingsSet -ExecutionTimeLimit (New-TimeSpan -Minutes 5) -AllowStartIfOnBatteries
Register-ScheduledTask -TaskName 'dori-mhc-host' -TaskPath '\' -Action $action -Trigger $trigger -Principal $principal -Settings $settings -Force | Out-Null
Start-ScheduledTask -TaskName 'dori-mhc-host' -TaskPath '\'
