#!/usr/bin/env bash
# Install cosmic-applet-ip system-wide. Run as root or via sudo.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$HERE"

NAME="cosmic-applet-ip"
APPID="io.cpknight.CosmicAppletIp"
PREFIX="${PREFIX:-/usr}"
SUDO=""

if [[ $EUID -ne 0 ]]; then
    SUDO="sudo"
fi

echo "==> Building release binary…"
cargo build --release

echo "==> Installing files to ${PREFIX}…"
$SUDO install -Dm0755 "target/release/${NAME}" "${PREFIX}/bin/${NAME}"
$SUDO install -Dm0644 "data/${APPID}.desktop" \
    "${PREFIX}/share/applications/${APPID}.desktop"
$SUDO install -Dm0644 "data/${APPID}.metainfo.xml" \
    "${PREFIX}/share/metainfo/${APPID}.metainfo.xml"
$SUDO install -Dm0644 "data/icons/${APPID}-symbolic.svg" \
    "${PREFIX}/share/icons/hicolor/symbolic/apps/${APPID}-symbolic.svg"

echo
echo "Done. Add the applet to your panel:"
echo "  Settings → Desktop → Panel → Configure panel applets → Add applet"
