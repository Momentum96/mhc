# Copies omo's anthropic-subscription entry (claude-cli accounts; access/refresh are the sentinel placeholder, not
# real tokens) into mhc's auth.json so mhc runs Claude on the account folders omo already uses on this machine.
$ErrorActionPreference = 'Stop'
$src = Join-Path $env:USERPROFILE '.omo\agent\auth.json'
$dst = Join-Path $env:USERPROFILE '.maho\home\.omo\agent\auth.json'
if (-not (Test-Path -LiteralPath $src)) { exit 0 }
$entry = (Get-Content -LiteralPath $src -Raw | ConvertFrom-Json).'anthropic-subscription'
if ($null -eq $entry) { exit 0 }
$accounts = @($entry.accounts | Where-Object { $_.store -eq 'claude-cli' })
if ($accounts.Count -eq 0) { exit 0 }
$entry.accounts = $accounts
$doc = if (Test-Path -LiteralPath $dst) { Get-Content -LiteralPath $dst -Raw | ConvertFrom-Json } else { [pscustomobject]@{} }
$current = if ($doc.PSObject.Properties.Name -contains 'anthropic-subscription') { $doc.'anthropic-subscription' | ConvertTo-Json -Depth 20 -Compress } else { '' }
$wanted = $entry | ConvertTo-Json -Depth 20 -Compress
if ($current -eq $wanted) { exit 0 }
$doc | Add-Member -NotePropertyName 'anthropic-subscription' -NotePropertyValue $entry -Force
$tmp = "$dst.tmp"
[IO.File]::WriteAllText($tmp, ($doc | ConvertTo-Json -Depth 20), (New-Object Text.UTF8Encoding $false))
Move-Item -LiteralPath $tmp -Destination $dst -Force
