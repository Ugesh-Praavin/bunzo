#!/usr/bin/env bash
set -euo pipefail

INSTALL_DIR="/usr/local/bin"
BASE_URL="https://sourceforge.net/projects/bunzo/files"

echo "Installing Bunzo..."

# Download binary
echo "Downloading bunzo..."
curl -fsSL "$BASE_URL/bunzo-linux-x86_64.tar.gz" -o /tmp/bunzo.tar.gz

# Extract
tar -xzf /tmp/bunzo.tar.gz -C /tmp

# Check sudo
if ! command -v sudo &>/dev/null; then
    echo "Error: sudo not found."
    exit 1
fi

# Install
sudo install -m 755 /tmp/bunzo "$INSTALL_DIR/bunzo"

# Cleanup
rm -f /tmp/bunzo.tar.gz /tmp/bunzo

# Verify
if ! command -v bunzo &>/dev/null; then
    echo "Warning: installed but not found in PATH."
    echo "Add $INSTALL_DIR to your PATH manually."
    exit 1
fi

echo "Done! Run: bunzo --help"