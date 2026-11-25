#!/usr/bin/env bash
# Installs a packaged mhc for Dori on a macOS or Linux worker: the package archive given as $1 goes to
# ~/.maho/dist/mhc, mhc-dori.sh beside this script becomes ~/.local/bin/mhc, the persistent launch spec is written,
# and the user's mhc engine host is registered to start at login (launchd ai.dori.mhc-session-host on macOS,
# systemd --user mhc-session-host.service on Linux) and started once. Run it again to upgrade.
set -euo pipefail
archive=${1:?usage: install-worker.sh <mhc-<target>.tar.gz>}
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
maho="$HOME/.maho"
mkdir -p "$maho/dist" "$HOME/.local/bin"
chmod 700 "$maho"
stage="$(mktemp -d "$maho/dist/.stage.XXXXXX")"
tar -xzf "$archive" -C "$stage"
top="$(find "$stage" -mindepth 1 -maxdepth 1 -type d | head -1)"
[ -x "$top/mhc" ] || { echo "install-worker: $archive has no mhc executable" >&2; exit 1; }
rm -rf "$maho/dist/mhc.old"
[ ! -d "$maho/dist/mhc" ] || mv "$maho/dist/mhc" "$maho/dist/mhc.old"
mv "$top" "$maho/dist/mhc"
rm -rf "$stage"
[ "$(uname -s)" != Darwin ] || bash "$here/mhc-sign-macos.sh" "$maho/dist/mhc"
install -m 755 "$here/mhc-dori.sh" "$HOME/.local/bin/mhc"
spec="$maho/dori-mhc-launch-spec.json"
printf '%s\n' '{"spec_version": 1, "core": {"session_runtime": "in-process", "multi_session": true, "extensions": []}, "tunables": {"idleExitMs": 900000, "coldStart": "persistent"}}' > "$spec"
chmod 600 "$spec"
"$HOME/.local/bin/mhc" --version
case "$(uname -s)" in
Darwin)
	plist="$HOME/Library/LaunchAgents/ai.dori.mhc-session-host.plist"
	cat > "$plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key><string>ai.dori.mhc-session-host</string>
  <key>ProgramArguments</key>
  <array><string>$HOME/.local/bin/mhc</string><string>host</string><string>ensure</string><string>--launch-spec</string><string>$spec</string><string>--json</string></array>
  <key>EnvironmentVariables</key><dict><key>PATH</key><string>$HOME/.local/bin:$HOME/.bun/bin:/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin</string></dict>
  <key>RunAtLoad</key><true/>
  <key>AbandonProcessGroup</key><true/>
  <key>ProcessType</key><string>Interactive</string>
</dict>
</plist>
EOF
	launchctl bootout "gui/$(id -u)/ai.dori.mhc-session-host" 2>/dev/null || true
	launchctl bootstrap "gui/$(id -u)" "$plist"
	;;
Linux)
	unit="$HOME/.config/systemd/user/mhc-session-host.service"
	mkdir -p "$(dirname "$unit")"
	cat > "$unit" <<EOF
[Unit]
Description=mhc session host (Dori)
StartLimitIntervalSec=0

[Service]
Type=oneshot
TimeoutStartSec=300
ExecStart=$HOME/.local/bin/mhc host ensure --launch-spec $spec --json
Environment="PATH=$HOME/.local/bin:$HOME/.bun/bin:/usr/local/bin:/usr/bin:/bin"
KillMode=process

[Install]
WantedBy=default.target
EOF
	systemctl --user daemon-reload
	systemctl --user enable mhc-session-host.service
	systemctl --user start mhc-session-host.service
	;;
esac
"$HOME/.local/bin/mhc" host status --json | tail -1 | head -c 300
echo
