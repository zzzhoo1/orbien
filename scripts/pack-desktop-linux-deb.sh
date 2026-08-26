#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN="${ROOT}/target/release/orbien-desktop"
VERSION=""
OUTDIR="${ROOT}/dist"
ARCH_LABEL=""
MAINTAINER="orbien <orbien@users.noreply.github.com>"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --bin) BIN="$2"; shift 2 ;;
    --version) VERSION="$2"; shift 2 ;;
    --outdir) OUTDIR="$2"; shift 2 ;;
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

DEB_VERSION="${VERSION//-SNAPSHOT/~SNAPSHOT}"
DEB_VERSION="${DEB_VERSION//-/\~}"

if [[ -z "$ARCH_LABEL" ]]; then
  case "$(uname -m)" in
    x86_64|amd64) ARCH_LABEL="amd64" ;;
    aarch64|arm64) ARCH_LABEL="arm64" ;;
    *) ARCH_LABEL="$(uname -m)" ;;
  esac
fi

if [[ ! -f "$BIN" ]]; then
  echo "binary not found: $BIN" >&2
  echo "build first: cargo build --release -p orbien-desktop" >&2
  exit 1
fi

ICON128="${ROOT}/desktop/assets/app-icons/128x128.png"
ICON64="${ROOT}/desktop/assets/app-icons/64x64.png"
ICON32="${ROOT}/desktop/assets/app-icons/32x32.png"
DESKTOP_SRC="${ROOT}/desktop/linux/orbien-desktop.desktop"

mkdir -p "$OUTDIR"
STAGE="${OUTDIR}/.deb-stage-$$"
PKG_ROOT="${STAGE}/orbien-desktop_${DEB_VERSION}_${ARCH_LABEL}"
rm -rf "$STAGE"
mkdir -p \
  "${PKG_ROOT}/DEBIAN" \
  "${PKG_ROOT}/usr/bin" \
  "${PKG_ROOT}/usr/share/applications" \
  "${PKG_ROOT}/usr/share/icons/hicolor/128x128/apps" \
  "${PKG_ROOT}/usr/share/icons/hicolor/64x64/apps" \
  "${PKG_ROOT}/usr/share/icons/hicolor/32x32/apps" \
  "${PKG_ROOT}/usr/share/doc/orbien-desktop"

install -m 755 "$BIN" "${PKG_ROOT}/usr/bin/orbien-desktop"

if [[ -f "$DESKTOP_SRC" ]]; then
  install -m 644 "$DESKTOP_SRC" "${PKG_ROOT}/usr/share/applications/orbien-desktop.desktop"
else
  cat > "${PKG_ROOT}/usr/share/applications/orbien-desktop.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=Orbien Desktop
Comment=Orbien intranet penetration desktop client
Exec=orbien-desktop
Icon=orbien-desktop
Terminal=false
Categories=Network;Utility;
StartupNotify=true
EOF
fi

[[ -f "$ICON128" ]] && install -m 644 "$ICON128" \
  "${PKG_ROOT}/usr/share/icons/hicolor/128x128/apps/orbien-desktop.png"
[[ -f "$ICON64" ]] && install -m 644 "$ICON64" \
  "${PKG_ROOT}/usr/share/icons/hicolor/64x64/apps/orbien-desktop.png"
[[ -f "$ICON32" ]] && install -m 644 "$ICON32" \
  "${PKG_ROOT}/usr/share/icons/hicolor/32x32/apps/orbien-desktop.png"

cat > "${PKG_ROOT}/usr/share/doc/orbien-desktop/copyright" <<EOF
Format: https://www.debian.org/doc/packaging-manuals/copyright-format/1.0/
Upstream-Name: orbien-desktop
Source: https://github.com/orbien-org/orbien

Files: *
Copyright: Orbien contributors
License: Apache-2.0
EOF

INSTALLED_SIZE="$(du -sk "$PKG_ROOT" | awk '{print $1}')"

cat > "${PKG_ROOT}/DEBIAN/control" <<EOF
Package: orbien-desktop
Version: ${DEB_VERSION}
Section: net
Priority: optional
Architecture: ${ARCH_LABEL}
Maintainer: ${MAINTAINER}
Installed-Size: ${INSTALLED_SIZE}
Depends: libc6, libxcb1, libxkbcommon0, libwayland-client0, libegl1
Description: Orbien Desktop client
 Native Slint GUI for the Orbien intranet-penetration client.
EOF

DEB_NAME="orbien-desktop_${VERSION}_linux_${ARCH_LABEL}.deb"
DEB_PATH="${OUTDIR}/${DEB_NAME}"
rm -f "$DEB_PATH"

if command -v dpkg-deb >/dev/null 2>&1; then
  dpkg-deb --build --root-owner-group "$PKG_ROOT" "$DEB_PATH"
else
  echo "dpkg-deb not found; creating ar archive manually" >&2
  (
    cd "$PKG_ROOT"
    tar czf "${STAGE}/data.tar.gz" usr
    (
      cd DEBIAN
      tar czf "${STAGE}/control.tar.gz" .
    )
    echo "2.0" > "${STAGE}/debian-binary"
    (
      cd "$STAGE"
      ar r "$DEB_PATH" debian-binary control.tar.gz data.tar.gz
    )
  )
fi

rm -rf "$STAGE"
echo "wrote ${DEB_PATH}"
ls -lh "$DEB_PATH"
