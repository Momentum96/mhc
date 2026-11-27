# One-time setup of the Windows mhc shadow home used by mhc-dori.cmd: junctions every directory of the real home
# into %USERPROFILE%\.maho\home except .omo, .maho, .pi and .senpi, so tools that read ~/.ssh, ~/.agents or
# ~/.config see the real ones. Safe to run again; existing entries are left alone.
$ErrorActionPreference = 'Stop'
$real = $env:USERPROFILE
$shadow = Join-Path $real '.maho\home'
New-Item -ItemType Directory -Force -Path (Join-Path $shadow '.omo\agent') | Out-Null
$skip = @('.omo', '.maho', '.pi', '.senpi')
Get-ChildItem -LiteralPath $real -Directory -Force | Where-Object { $skip -notcontains $_.Name } | ForEach-Object {
    $link = Join-Path $shadow $_.Name
    if (-not (Test-Path -LiteralPath $link)) {
        New-Item -ItemType Junction -Path $link -Target $_.FullName | Out-Null
    }
}
$memory = Join-Path $shadow '.omo\memory'
if (-not (Test-Path -LiteralPath $memory)) {
    New-Item -ItemType Junction -Path $memory -Target (Join-Path $real '.omo\memory') | Out-Null
}
Write-Output "mhc shadow home ready: $shadow"
