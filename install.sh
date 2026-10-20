#!/usr/bin/env bash
set -e

# Xorin Player One-Line Universal Linux & macOS Installer
# Usage: curl -fsSL https://raw.githubusercontent.com/shafayeatt1995/music_player/main/install.sh | bash

REPO="shafayeatt1995/music_player"
OS="$(uname -s)"
ARCH="$(uname -m)"

echo "🎵 Installing Xorin Player for $OS ($ARCH)..."

# Detect OS
if [ "$OS" = "Darwin" ]; then
    echo "🍏 macOS detected."
    if command -v brew >/dev/null 2>&1; then
        echo "Installing via Homebrew..."
        brew install --cask https://raw.githubusercontent.com/${REPO}/main/packaging/homebrew/xorin-player.rb
    else
        echo "Downloading macOS App bundle..."
        curl -sL "https://github.com/${REPO}/releases/latest/download/xorin-player-macos-app.zip" -o /tmp/xorin-player.zip
        unzip -q /tmp/xorin-player.zip -d /tmp/
        rm -rf "/Applications/Xorin Player.app"
        mv "/tmp/Xorin Player.app" /Applications/
        xattr -d com.apple.quarantine "/Applications/Xorin Player.app" 2>/dev/null || true
        echo "✅ Installed to /Applications/Xorin Player.app!"
    fi
elif [ "$OS" = "Linux" ]; then
    echo "🐧 Linux detected."
    if command -v apt >/dev/null 2>&1; then
        DEB_ARCH="amd64"
        if [ "$ARCH" = "aarch64" ] || [ "$ARCH" = "arm64" ]; then
            DEB_ARCH="arm64"
        fi
        echo "Downloading Debian package (.deb)..."
        curl -sL "https://github.com/${REPO}/releases/latest/download/xorin-player_1.0.0_${DEB_ARCH}.deb" -o /tmp/xorin.deb
        sudo apt-get update -y || true
        sudo apt-get install -y /tmp/xorin.deb
        rm -f /tmp/xorin.deb
        echo "✅ Installed successfully! You can find Xorin Player in your app menu or run 'xorin-player'."
    else
        echo "Installing standalone binary..."
        curl -sL "https://github.com/${REPO}/releases/latest/download/xorin-player-linux-${ARCH}.tar.gz" -o /tmp/xorin.tar.gz
        tar -xzf /tmp/xorin.tar.gz -C /tmp/
        sudo mv /tmp/xorin-player-linux-${ARCH} /usr/local/bin/xorin-player
        sudo chmod +x /usr/local/bin/xorin-player
        echo "✅ Installed to /usr/local/bin/xorin-player!"
    fi
fi

echo "🚀 Xorin Player is ready to use!"
