#!/usr/bin/env bash
set -e

echo "Installing Bunzo Compiler (bzc)..."

# Default install directory
INSTALL_DIR="/usr/local/bin"

# Check if we have root privileges
if [ "$EUID" -ne 0 ]; then
  echo "Not running as root. Installing to ~/.local/bin instead..."
  INSTALL_DIR="$HOME/.local/bin"
  mkdir -p "$INSTALL_DIR"
fi

echo "Copying bzc to $INSTALL_DIR..."
cp bzc "$INSTALL_DIR/"
chmod +x "$INSTALL_DIR/bzc"

echo "================================================"
echo "Bunzo compiler successfully installed to $INSTALL_DIR/bzc"
echo "You can test it by running: bzc --help"
echo "================================================"
