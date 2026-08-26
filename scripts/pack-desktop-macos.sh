#!/usr/bin/env bash

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN="${ROOT}/target/release/orbien-desktop"
VERSION=""
OUTDIR="${ROOT}/dist"
APP_NAME="Orbien Desktop"
ARCH_LABEL=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --bin) BIN="$2"; shift 2 ;;
    --version) VERSION="$2"; shift 2 ;;
    --outdir) OUTDIR="$2"; shift 2 ;;
    --name) APP_NAME="$2"; shift 2 ;;
    --arch) ARCH_LABEL="$2"; shift 2 ;;
    *) echo "unknown arg: $1" >&2; exit 1 ;;
  esac
done

if [[ -z "$VERSION" ]]; then
  VERSION="$(
    cargo metadata --no-deps --format-version 1 --manifest-path "${ROOT}/Cargo.toml" \
      | python3 -c '
import json, sys
meta = json.load(sys.stdin)
for p in meta["packages"]:
    if p["name"] == "orbien-desktop":
        print(p["version"])
        break
else:
    sys.exit("orbien-desktop not found in cargo metadata")
'
  )"
fi

if [[ -z "$ARCH_LABEL" ]]; then
  case "$(uname -m)" in
    arm64|aarch64) ARCH_LABEL="arm64" ;;
    x86_64|amd64) ARCH_LABEL="amd64" ;;
    *) ARCH_LABEL="$(uname -m)" ;;
  esac
fi

if [[ ! -f "$BIN" ]]; then
  echo "binary not found: $BIN" >&2
  echo "build first: cargo build --release -p orbien-desktop" >&2
  exit 1
fi

PLIST_SRC="${ROOT}/desktop/macos/Info.plist"
ICON_SRC="${ROOT}/desktop/assets/app-icons/icon.icns"
if [[ ! -f "$PLIST_SRC" ]]; then
  echo "missing Info.plist: $PLIST_SRC" >&2
  exit 1
fi

mkdir -p "$OUTDIR"
APP_DIR="${OUTDIR}/${APP_NAME}.app"
CONTENTS="${APP_DIR}/Contents"
MACOS="${CONTENTS}/MacOS"
RESOURCES="${CONTENTS}/Resources"

rm -rf "$APP_DIR"
mkdir -p "$MACOS" "$RESOURCES"

cp "$BIN" "${MACOS}/orbien-desktop"
chmod +x "${MACOS}/orbien-desktop"

PLIST_DST="${CONTENTS}/Info.plist"
cp "$PLIST_SRC" "$PLIST_DST"
/usr/libexec/PlistBuddy -c "Set :CFBundleShortVersionString ${VERSION}" "$PLIST_DST"
/usr/libexec/PlistBuddy -c "Set :CFBundleVersion ${VERSION}" "$PLIST_DST"

if [[ -f "$ICON_SRC" ]]; then
  cp "$ICON_SRC" "${RESOURCES}/icon.icns"
fi

echo "wrote ${APP_DIR} (version ${VERSION})"

STAGE="${OUTDIR}/.dmg-stage-$$"
DMG_NAME="orbien-desktop_${VERSION}_darwin_${ARCH_LABEL}.dmg"
DMG_PATH="${OUTDIR}/${DMG_NAME}"
rm -rf "$STAGE"
mkdir -p "$STAGE"
cp -R "$APP_DIR" "$STAGE/"
ln -s /Applications "${STAGE}/Applications"

rm -f "$DMG_PATH"
hdiutil create \
  -volname "${APP_NAME}" \
  -srcfolder "$STAGE" \
  -ov \
  -format UDZO \
  "$DMG_PATH" >/dev/null
rm -rf "$STAGE"

echo "wrote ${DMG_PATH}"
ls -lh "$APP_DIR" "$DMG_PATH"
