#!/usr/bin/env bash
set -euo pipefail

INSTALL_DIR="/usr/local/bin"
BASE_URL="https://downloads.sourceforge.net/project/bunzo"

echo "Installing Bunzo..."

# Download binary
echo "Downloading bunzo..."
curl -fsSL "$BASE_URL/bunzo-linux-x86_64.tar.gz" -o /tmp/bunzo.tar.gz

# Extract
mkdir -p /tmp/bunzo-extract
tar -xzf /tmp/bunzo.tar.gz -C /tmp/bunzo-extract

# Check sudo
if ! command -v sudo &>/dev/null; then
    echo "Error: sudo not found."
    exit 1
fi

# Install
sudo install -m 755 /tmp/bunzo-extract/bzc "$INSTALL_DIR/bzc"
sudo mkdir -p "$INSTALL_DIR/runtime"
sudo cp -r /tmp/bunzo-extract/runtime/* "$INSTALL_DIR/runtime/"

# Cleanup
rm -rf /tmp/bunzo.tar.gz /tmp/bunzo-extract

# Verify
if ! command -v bzc &>/dev/null; then
    echo "Warning: installed but not found in PATH."
    echo "Add $INSTALL_DIR to your PATH manually."
    exit 1
fi

echo "Done! Run: bzc --help"