#!/usr/bin/env bash
# Runs mhc beside an OmO install on the same account without touching OmO's state.
#
# mhc reads and writes many `~/.omo` paths (agent dir, omo.jsonc, migration journal, memory,
# team registry, context advice, host sockets). Instead of overriding each one, this wrapper
# runs mhc with HOME pointed at a shadow home, ~/.maho/home: every entry of the real home is a
# symlink there except `.omo`, which is mhc's own directory, and the legacy `.pi`/`.senpi` homes, which
# mhc's startup migrations would otherwise import from. Claude CLI accounts are the one
# shared piece: MAHO_CLAUDE_ACCOUNT_ROOT points at OmO's account dirs, because the claude CLI
# keys its Keychain login on the exact config-dir path.
#
# Environment: MHC_BIN (default: this checkout's packaged dist/mhc/mhc, built by tools/package-native.mjs).
set -euo pipefail

real_home="$(eval echo "~$(id -un)")"
shadow="$real_home/.maho/home"
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
mhc_bin="${MHC_BIN:-$script_dir/../../dist/mhc/mhc}"

mkdir -p "$shadow/.omo/agent"
chmod 700 "$real_home/.maho" "$shadow" "$shadow/.omo"

shopt -s dotglob nullglob
for entry in "$real_home"/*; do
	name="${entry##*/}"
	case "$name" in .omo | .maho | .pi | .senpi) continue ;; esac
	[ -e "$shadow/$name" ] || [ -L "$shadow/$name" ] || ln -s "$entry" "$shadow/$name"
done
shopt -u dotglob nullglob

# Seed mhc's own config from OmO once; after that mhc's copies are its own.
[ -e "$shadow/.omo/omo.jsonc" ] || cp "$real_home/.omo/omo.jsonc" "$shadow/.omo/omo.jsonc"
for file in settings.json models.json auth.json; do
	if [ ! -e "$shadow/.omo/agent/$file" ] && [ -e "$real_home/.omo/agent/$file" ]; then
		cp "$real_home/.omo/agent/$file" "$shadow/.omo/agent/$file"
	fi
done
[ ! -e "$shadow/.omo/agent/auth.json" ] || chmod 600 "$shadow/.omo/agent/auth.json"

# A caller running inside an OmO session passes its session identity and agent dir in OMO_*,
# SENPI_* and PI_* variables; none of them may reach mhc.
for name in $(compgen -e); do
	case "$name" in OMO_* | SENPI_* | PI_* | MAHO_*) unset "$name" ;; esac
done

export HOME="$shadow"
export MAHO_CLAUDE_ACCOUNT_ROOT="$real_home/.omo/agent/anthropic-subscription-claude"
exec "$mhc_bin" "$@"
