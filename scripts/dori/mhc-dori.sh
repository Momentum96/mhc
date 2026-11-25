#!/usr/bin/env bash
# mhc (Rust omo port) side-install wrapper for Dori. Runs mhc with HOME=~/.maho/home: every real home entry is a
# symlink there except .omo, .maho and the legacy .pi/.senpi homes, so mhc keeps its own agent dir, sessions,
# host sockets and ChatGPT login. No ChatGPT tokens are copied from the real ~/.omo, so mhc never refreshes omo's.
# Shared with omo (2026-11-25 owner decision): durable memory (~/.omo/memory), omo.jsonc and the global
# agent/AGENTS.md are symlinks to the real ones, so a Dori session on either engine reads and writes one memory.
# Claude runs through the claude CLI on the same account folders omo uses on this machine (owner decision 2026-11-25):
# MAHO_CLAUDE_ACCOUNT_ROOT is omo's, and omo's anthropic-subscription entry is copied into mhc's auth.json on every
# start; for claude-cli accounts its access/refresh are the sentinel placeholder, so no real token is duplicated. ChatGPT stays a separate mhc login because mhc refreshes those tokens itself.
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
if command -v python3 >/dev/null 2>&1 && [ -f "$real_home/.omo/agent/auth.json" ]; then
	python3 -I - "$real_home/.omo/agent/auth.json" "$shadow/.omo/agent/auth.json" <<'PY' || echo "mhc-dori: could not sync Claude accounts from omo" >&2
import json, os, sys
src, dst = sys.argv[1], sys.argv[2]
entry = json.load(open(src)).get("anthropic-subscription") or {}
accounts = [a for a in entry.get("accounts") or [] if a.get("store") == "claude-cli"]
if accounts:
    doc = json.load(open(dst)) if os.path.exists(dst) else {}
    shared = {**entry, "accounts": accounts}
    if doc.get("anthropic-subscription") != shared:
        doc["anthropic-subscription"] = shared
        tmp = dst + ".tmp"
        with open(os.open(tmp, os.O_WRONLY | os.O_CREAT | os.O_TRUNC, 0o600), "w") as f:
            json.dump(doc, f, indent=2)
        os.replace(tmp, dst)
PY
fi
export MAHO_CLAUDE_ACCOUNT_ROOT="$real_home/.omo/agent/anthropic-subscription-claude"
exec "$mhc_bin" "$@"
