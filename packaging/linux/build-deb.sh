#!/usr/bin/env bash
set -e

# Build a Debian (.deb) package for Xorin Player
# Usage: ./build-deb.sh <binary_path> <version> <arch> <output_dir>

BINARY_PATH="${1:-target/release/xorin-player}"
VERSION="${2:-1.0.0}"
ARCH="${3:-amd64}"
OUTPUT_DIR="${4:-dist}"

PKG_NAME="xorin-player"
PKG_DIR="/tmp/${PKG_NAME}_${VERSION}_${ARCH}"

echo "==> Building ${PKG_NAME}_${VERSION}_${ARCH}.deb..."
rm -rf "$PKG_DIR"
mkdir -p "$PKG_DIR/DEBIAN"
mkdir -p "$PKG_DIR/usr/bin"
mkdir -p "$PKG_DIR/usr/share/applications"
mkdir -p "$PKG_DIR/usr/share/icons/hicolor/256x256/apps"

# 1. Binary
cp "$BINARY_PATH" "$PKG_DIR/usr/bin/xorin-player"
chmod 755 "$PKG_DIR/usr/bin/xorin-player"

# 2. Desktop file (Menu & Right-click "Open With" associations)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cp "$SCRIPT_DIR/xorin-player.desktop" "$PKG_DIR/usr/share/applications/"

# 3. Icon
if [ -f "$SCRIPT_DIR/../../dark-logo.png" ]; then
    cp "$SCRIPT_DIR/../../dark-logo.png" "$PKG_DIR/usr/share/icons/hicolor/256x256/apps/xorin-player.png"
fi

# 4. Control file
cat <<EOF > "$PKG_DIR/DEBIAN/control"
Package: ${PKG_NAME}
Version: ${VERSION}
Section: sound
Priority: optional
Architecture: ${ARCH}
Depends: libasound2, libx11-6, libxcursor1, libxrandr2, libxi6
Maintainer: Xorin Team <shafayeat.anik@gmail.com>
Description: High-Fidelity Audio & Cinema Video Player
 Modern lightweight cross-platform media player powered by Rust, egui,
 Poweramp-grade acoustics, and high-definition cinema video decoding.
EOF

# 5. Build package
mkdir -p "$OUTPUT_DIR"
dpkg-deb --build --root-owner-group "$PKG_DIR" "$OUTPUT_DIR/${PKG_NAME}_${VERSION}_${ARCH}.deb"

echo "==> Successfully created $OUTPUT_DIR/${PKG_NAME}_${VERSION}_${ARCH}.deb"
rm -rf "$PKG_DIR"
