#!/usr/bin/env bash

set -e

if [[ "$(uname -s)" != "Linux" ]]; then
  echo "This installer only supports Linux."
  exit 1
fi

ARCH=$(uname -m)
case "$ARCH" in
  x86_64) PLATFORM="x86_64" ;;
  aarch64 | arm64) PLATFORM="arm64" ;;
  *)
    echo "Unsupported architecture: $ARCH"
    exit 1
    ;;
esac

URL="https://github.com/kevinj045/lulu/releases/latest/download/lulu-linux_${PLATFORM}.tar.gz"

TMP_DIR=$(mktemp -d)
cd "$TMP_DIR"

echo "Downloading lulu for $PLATFORM..."
curl -L -o lulu.tar.gz "$URL"

echo "Extracting..."
tar -xzf lulu.tar.gz

INSTALL_DIR="/usr/local/bin"
if [[ $EUID -ne 0 ]]; then
  echo "Installing to $INSTALL_DIR requires root privileges. Asking for sudo..."
  sudo mv lulu "$INSTALL_DIR/lulu"
else
  mv lulu "$INSTALL_DIR/lulu"
fi

chmod +x "$INSTALL_DIR/lulu"

echo "✅ lulu installed to $INSTALL_DIR/lulu"
lulu --version || echo "Try restarting your shell if 'lulu' is not yet in PATH."