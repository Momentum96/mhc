#!/usr/bin/env bash
# mhc (Rust omo port) side-install wrapper for Dori. Runs mhc with HOME=~/.maho/home: every real home entry is a
# symlink there except .omo, .maho and the legacy .pi/.senpi homes, so mhc keeps its own agent dir, sessions,
# host sockets and logins. Own login only: nothing is copied from the real ~/.omo (no auth.json seeding), and the
# Claude CLI account root is the shadow home's own, so the owner's real tokens are never refreshed.
# Shared with omo (2026-11-25 owner decision): durable memory (~/.omo/memory), omo.jsonc and the global
# agent/AGENTS.md are symlinks to the real ones, so a Dori session on either engine reads and writes one memory.
# Remove everything: rm -rf ~/.maho ~/.local/bin/mhc
set -euo pipefail
real_home="$(eval echo "~$(id -un)")"
shadow="$real_home/.maho/home"
mhc_bin="${MHC_BIN:-$real_home/.maho/dist/mhc/mhc}"
mkdir -p "$shadow/.omo/agent"
chmod 700 "$real_home/.maho" "$shadow" "$shadow/.omo"
shopt -s dotglob nullglob
for entry in "$real_home"/*; do
	name="${entry##*/}"
	case "$name" in .omo | .maho | .pi | .senpi) continue ;; esac
	[ -e "$shadow/$name" ] || [ -L "$shadow/$name" ] || ln -s "$entry" "$shadow/$name"
done
shopt -u dotglob nullglob
for rel in memory omo.jsonc agent/AGENTS.md; do
	src="$real_home/.omo/$rel"
	dst="$shadow/.omo/$rel"
	[ -e "$src" ] && [ ! -L "$dst" ] || continue
	[ ! -e "$dst" ] || mv "$dst" "$dst.shadow-bak-$(date +%Y%m%d%H%M%S)"
	ln -s "$src" "$dst"
done
for name in $(compgen -e); do
	case "$name" in OMO_* | SENPI_* | PI_* | MAHO_*) unset "$name" ;; esac
done
export HOME="$shadow"
export PATH="$real_home/.local/bin:$real_home/.bun/bin:$PATH"
export MAHO_CLAUDE_ACCOUNT_ROOT="$shadow/.omo/agent/anthropic-subscription-claude"
exec "$mhc_bin" "$@"
