#!/usr/bin/env bash
set -e

# Setup script for Xorin Player APT Repository
# Enables direct 'sudo apt install xorin-player' on Debian/Ubuntu/Mint/Pop!_OS

REPO_URL="https://shafayeatt1995.github.io/music_player/apt"
KEY_URL="https://shafayeatt1995.github.io/music_player/apt/xorin-player.gpg"

echo "🎵 Setting up Xorin Player APT Repository..."

# Ensure curl and gnupg are installed
if ! command -v curl >/dev/null 2>&1 || ! command -v gpg >/dev/null 2>&1; then
    sudo apt-get update -y
    sudo apt-get install -y curl gnupg
fi

# Add GPG key
sudo mkdir -p /etc/apt/keyrings
curl -fsSL "$KEY_URL" | sudo gpg --dearmor -o /etc/apt/keyrings/xorin-player.gpg 2>/dev/null || true

# Add repository source
echo "deb [arch=amd64,arm64 signed-by=/etc/apt/keyrings/xorin-player.gpg] $REPO_URL stable main" | sudo tee /etc/apt/sources.list.d/xorin-player.list > /dev/null

# Update package lists
sudo apt-get update -q

echo "✅ Xorin Player repository added successfully!"
echo "👉 You can now install Xorin Player anytime by running:"
echo "   sudo apt install xorin-player"
