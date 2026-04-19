#!/usr/bin/env bash
# Remove a previously-installed cosmic-applet-ip. Run as root or via sudo.
set -euo pipefail

NAME="cosmic-applet-ip"
APPID="com.cpknight.CosmicAppletIp"
PREFIX="${PREFIX:-/usr}"
SUDO=""

if [[ $EUID -ne 0 ]]; then
    SUDO="sudo"
fi

for f in \
    "${PREFIX}/bin/${NAME}" \
    "${PREFIX}/share/applications/${APPID}.desktop" \
    "${PREFIX}/share/metainfo/${APPID}.metainfo.xml" \
    "${PREFIX}/share/icons/hicolor/symbolic/apps/${APPID}-symbolic.svg"; do
    if [[ -e "$f" ]]; then
        echo "rm $f"
        $SUDO rm -f "$f"
    fi
done

echo "Uninstalled."
