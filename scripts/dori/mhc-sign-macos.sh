#!/usr/bin/env bash
# Signs the installed mhc executables with one local, self-signed code-signing identity and fixed identifiers.
# An ad-hoc signature changes identity with every build, so macOS privacy prompts (for example the removable-volume
# prompt for /Volumes/Work on macmini) return after each upgrade; a stable signature keeps the approval.
# The identity lives in its own keychain under ~/.maho/signing, created on first use; its password never leaves
# that directory.
set -euo pipefail
dist="${1:-$HOME/.maho/dist/mhc}"
dir="$HOME/.maho/signing"
kc="$dir/mhc-signing.keychain-db"
name="Dori mhc local signing"
if [ ! -f "$kc" ]; then
	mkdir -p "$dir"
	chmod 700 "$dir"
	umask 077
	openssl rand -hex 24 > "$dir/keychain.pw"
	work="$(mktemp -d "$dir/.new.XXXXXX")"
	cat > "$work/req.cnf" <<EOF
[req]
distinguished_name = dn
x509_extensions = ext
prompt = no
[dn]
CN = $name
[ext]
basicConstraints = critical,CA:false
keyUsage = critical,digitalSignature
extendedKeyUsage = critical,codeSigning
EOF
	openssl req -x509 -newkey rsa:2048 -nodes -days 7300 -config "$work/req.cnf" -keyout "$work/key.pem" -out "$work/cert.pem" 2>/dev/null
	openssl pkcs12 -export -inkey "$work/key.pem" -in "$work/cert.pem" -name "$name" -out "$work/id.p12" -passout "file:$dir/keychain.pw"
	security create-keychain -p "$(cat "$dir/keychain.pw")" "$kc"
	security set-keychain-settings "$kc"
	security unlock-keychain -p "$(cat "$dir/keychain.pw")" "$kc"
	security import "$work/id.p12" -k "$kc" -P "$(cat "$dir/keychain.pw")" -T /usr/bin/codesign >/dev/null
	security set-key-partition-list -S apple-tool:,apple:,codesign: -s -k "$(cat "$dir/keychain.pw")" "$kc" >/dev/null
	rm -rf "$work"
fi
security unlock-keychain -p "$(cat "$dir/keychain.pw")" "$kc"
for f in "$dist"/*; do
	[ -f "$f" ] && [ -x "$f" ] && file -b "$f" | grep -q '^Mach-O' || continue
	codesign --force --keychain "$kc" --sign "$name" --identifier "ai.dori.mhc.$(basename "$f")" "$f"
done
security lock-keychain "$kc"
codesign -dv "$dist/mhc" 2>&1 | grep -E '^(Identifier|Authority)='
