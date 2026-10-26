#!/usr/bin/env bash
# Generate latest.json for the Tauri updater and (optionally) create the
# GitHub release with the signed installer.
#
# Prerequisites:
#   - .tauri/themeswitcher.key   (private key, NEVER commit)
#   - built installer in src-tauri/target/release/bundle/nsis/
#
# Usage:
#   bash scripts/make_release.sh <version> <path-to-setup.exe>

set -euo pipefail

VERSION="${1:-1.0.0}"
INSTALLER="${2:-src-tauri/target/release/bundle/nsis/Windows Theme Switcher_${VERSION}_x64-setup.exe}"
KEY=".tauri/themeswitcher.key"
REPO="DashAkoul/windows-theme-switcher"
TAG="v${VERSION}"

cd "$(dirname "$0")/.."

if [ ! -f "$INSTALLER" ]; then
    echo "ERROR: installer not found: $INSTALLER"
    exit 1
fi

echo "==> Signing installer with minisign key"
npx tauri signer sign -f "$KEY" -p "" "$INSTALLER" > /dev/null
SIG_FILE="${INSTALLER}.sig"
if [ ! -f "$SIG_FILE" ]; then
    echo "ERROR: signature file not produced: $SIG_FILE"
    exit 1
fi
SIGNATURE=$(tr -d '\n' < "$SIG_FILE")
echo "signature: ${SIGNATURE:0:40}... (${#SIGNATURE} chars)"

ASSET_NAME=$(basename "$INSTALLER")
# GitHub replaces spaces with dots in uploaded asset names, so the download
# URL must use that sanitized name (a %20-encoded URL returns 404).
ASSET_URL_NAME=$(python -c "import sys; print(sys.argv[1].replace(' ', '.'))" "$ASSET_NAME")
URL="https://github.com/${REPO}/releases/download/${TAG}/${ASSET_URL_NAME}"
PUB_DATE=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

cat > latest.json <<EOF
{
  "version": "${VERSION}",
  "notes": "Windows Theme Switcher ${VERSION}",
  "pub_date": "${PUB_DATE}",
  "platforms": {
    "windows-x86_64": {
      "url": "${URL}",
      "signature": "${SIGNATURE}"
    }
  }
}
EOF

echo "==> latest.json written:"
cat latest.json

if [ "${3:-}" = "--release" ]; then
    echo "==> Creating GitHub release ${TAG}"
    gh release create "$TAG" "$INSTALLER" latest.json \
        --title "Windows Theme Switcher ${VERSION}" \
        --notes "One-click Windows Light/Dark theme switcher (system tray app)." \
        --repo "$REPO"
    echo "==> Release created: https://github.com/${REPO}/releases/tag/${TAG}"
fi
