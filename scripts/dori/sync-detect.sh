#!/usr/bin/env bash
# Step 1 of docs/dori-sync/backlog.md section 6: report the senpi and omo ranges between the PINS.md
# pins and the newest omo release. Exit 0 when the pins already match that release, 10 when a sync
# is due, anything else on error. Prints one KEY=value line per fact for scripts to read.
set -euo pipefail

repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
upstream="${UPSTREAM_DIR:-$(dirname "$repo")/upstream}"
senpi="$upstream/senpi"
omo="$upstream/oh-my-openagent"

pin_of() {
	awk -F'|' -v name="$1" '$2 ~ "^ *" name " *$" { gsub(/ /, "", $4); print $4; exit }' "$repo/PINS.md"
}

senpi_pin="$(pin_of senpi)"
omo_pin="$(pin_of oh-my-openagent)"
[ -n "$senpi_pin" ] && [ -n "$omo_pin" ] || { echo "ERROR=pins not found in PINS.md" >&2; exit 2; }

git -C "$senpi" fetch -q --tags origin
git -C "$omo" fetch -q --tags origin

omo_latest="$(git -C "$omo" tag --list 'v[0-9]*.[0-9]*.[0-9]*' --sort=-v:refname | head -1)"
senpi_version="$(git -C "$omo" show "$omo_latest:package.json" |
	sed -n 's/.*"@code-yeongyu\/senpi": *"\([^"]*\)".*/\1/p' | head -1)"
senpi_target="v$senpi_version"
git -C "$senpi" rev-parse -q --verify "$senpi_target^{commit}" >/dev/null ||
	{ echo "ERROR=senpi tag $senpi_target missing" >&2; exit 2; }

omo_pin_tag="$(git -C "$omo" describe --tags --exact-match "$omo_pin" 2>/dev/null || echo "$omo_pin")"

echo "OMO_LATEST=$omo_latest"
echo "OMO_PIN=$omo_pin_tag"
echo "SENPI_TARGET=$senpi_target"
echo "SENPI_PIN=$senpi_pin"
echo "OMO_COMMITS=$(git -C "$omo" rev-list --count "$omo_pin..$omo_latest")"
echo "SENPI_COMMITS=$(git -C "$senpi" rev-list --count "$senpi_pin..$senpi_target")"

if [ "$(git -C "$omo" rev-parse "$omo_pin^{commit}")" = "$(git -C "$omo" rev-parse "$omo_latest^{commit}")" ]; then
	echo "STATUS=up-to-date"
	exit 0
fi
echo "STATUS=sync-due"
exit 10
