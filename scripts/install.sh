#!/usr/bin/env bash
set -e

echo "Installing Bunzo..."

sudo install -m 755 bunzo /usr/local/bin/bunzo

echo "Done!"
echo "Run: bunzo --help"