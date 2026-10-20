#!/usr/bin/env bash
set -e

# Package Xorin Player as a native macOS App Bundle (.app)
# Usage: ./bundle-macos.sh <binary_path> <output_dir>

BINARY_PATH="${1:-target/release/xorin-player}"
OUTPUT_DIR="${2:-dist}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

APP_NAME="Xorin Player.app"
APP_DIR="$OUTPUT_DIR/$APP_NAME"

echo "==> Creating macOS App Bundle at $APP_DIR..."
rm -rf "$APP_DIR"
mkdir -p "$APP_DIR/Contents/MacOS"
mkdir -p "$APP_DIR/Contents/Resources"

# 1. Executable
cp "$BINARY_PATH" "$APP_DIR/Contents/MacOS/xorin-player"
chmod +x "$APP_DIR/Contents/MacOS/xorin-player"

# 2. Info.plist (Document types & MIME associations)
cp "$SCRIPT_DIR/Info.plist" "$APP_DIR/Contents/"

# 3. App Icon
if [ -f "$PROJECT_ROOT/logo.ico" ]; then
    # Create icns if sips and iconutil are available on macOS
    if command -v iconutil >/dev/null 2>&1 && [ -f "$PROJECT_ROOT/dark-logo.png" ]; then
        ICONSET="/tmp/Xorin.iconset"
        mkdir -p "$ICONSET"
        sips -z 16 16     "$PROJECT_ROOT/dark-logo.png" --out "$ICONSET/icon_16x16.png" >/dev/null 2>&1 || true
        sips -z 32 32     "$PROJECT_ROOT/dark-logo.png" --out "$ICONSET/icon_16x16@2x.png" >/dev/null 2>&1 || true
        sips -z 32 32     "$PROJECT_ROOT/dark-logo.png" --out "$ICONSET/icon_32x32.png" >/dev/null 2>&1 || true
        sips -z 64 64     "$PROJECT_ROOT/dark-logo.png" --out "$ICONSET/icon_32x32@2x.png" >/dev/null 2>&1 || true
        sips -z 128 128   "$PROJECT_ROOT/dark-logo.png" --out "$ICONSET/icon_128x128.png" >/dev/null 2>&1 || true
        sips -z 256 256   "$PROJECT_ROOT/dark-logo.png" --out "$ICONSET/icon_128x128@2x.png" >/dev/null 2>&1 || true
        sips -z 256 256   "$PROJECT_ROOT/dark-logo.png" --out "$ICONSET/icon_256x256.png" >/dev/null 2>&1 || true
        sips -z 512 512   "$PROJECT_ROOT/dark-logo.png" --out "$ICONSET/icon_256x256@2x.png" >/dev/null 2>&1 || true
        sips -z 512 512   "$PROJECT_ROOT/dark-logo.png" --out "$ICONSET/icon_512x512.png" >/dev/null 2>&1 || true
        sips -z 1024 1024 "$PROJECT_ROOT/dark-logo.png" --out "$ICONSET/icon_512x512@2x.png" >/dev/null 2>&1 || true
        iconutil -c icns "$ICONSET" -o "$APP_DIR/Contents/Resources/AppIcon.icns" || true
        rm -rf "$ICONSET"
    fi
fi

# 4. Zip bundle for distribution
cd "$OUTPUT_DIR"
zip -r -y "xorin-player-macos-app.zip" "$APP_NAME"

# 5. Create native macOS Disk Image (.dmg) with Applications symlink
if command -v hdiutil >/dev/null 2>&1; then
    echo "==> Creating macOS Disk Image (DMG)..."
    DMG_DIR="/tmp/xorin-dmg-source"
    rm -rf "$DMG_DIR"
    mkdir -p "$DMG_DIR"
    cp -R "$APP_NAME" "$DMG_DIR/"
    ln -s /Applications "$DMG_DIR/Applications"

    rm -f "xorin-player-macos-universal.dmg"
    hdiutil create -volname "Xorin Player" \
        -srcfolder "$DMG_DIR" \
        -ov -format UDZO \
        "xorin-player-macos-universal.dmg"
    rm -rf "$DMG_DIR"
    echo "==> Successfully created xorin-player-macos-universal.dmg"
fi

echo "==> macOS packaging complete: $APP_DIR, xorin-player-macos-app.zip, and xorin-player-macos-universal.dmg"
