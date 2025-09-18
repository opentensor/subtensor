#!/bin/bash

echo ""
echo "######################################################################"
echo "###     WARNING: DO NOT MODIFY THIS SCRIPT UNLESS YOU KNOW WHY!    ###"
echo "###                                                                ###"
echo "### This script is used by:                                        ###"
echo "###   • .github/workflows/docker-localnet.yml                      ###"
echo "###   • Dockerfile-localnet                                        ###"
echo "###                                                                ###"
echo "### Any changes may break CI builds or local Docker environments.  ###"
echo "######################################################################"
echo ""

set -e

echo "[*] Detecting platform..."
UNAME_OUT="$(uname -s)"
case "${UNAME_OUT}" in
    Linux*)     OS=Linux;;
    Darwin*)    OS=Mac;;
    *)          OS="UNKNOWN:${UNAME_OUT}"
esac

echo "[+] Platform: $OS"

# Determine if we have root privileges
if [ "$(id -u)" -eq 0 ]; then
  SUDO=""
else
  if command -v sudo &>/dev/null; then
    SUDO="sudo"
  else
    SUDO=""
  fi
fi

# Linux system dependencies
if [ "$OS" = "Linux" ]; then
    echo "[+] Installing dependencies on Linux..."

    if [ -z "$SUDO" ] && [ "$(id -u)" -ne 0 ]; then
        echo "[!] Warning: No sudo and not root. Skipping apt install."
    else
        $SUDO sed -i 's|http://archive.ubuntu.com/ubuntu|http://mirrors.edge.kernel.org/ubuntu|g' /etc/apt/sources.list || true
        $SUDO apt-get update
        $SUDO apt-get install -y --no-install-recommends \
            curl build-essential protobuf-compiler clang git pkg-config libssl-dev llvm libudev-dev
    fi

# macOS system dependencies
elif [ "$OS" = "Mac" ]; then
    echo "[+] Installing dependencies on macOS..."

    # Check if brew is installed
    if ! command -v brew &> /dev/null; then
        echo "[!] Homebrew not found. Installing..."
        /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
        eval "$(/opt/homebrew/bin/brew shellenv)"
    fi

    brew install protobuf openssl llvm pkg-config

    LDFLAGS="-L$(brew --prefix openssl)/lib"
    export LDFLAGS

    CPPFLAGS="-I$(brew --prefix openssl)/include"
    export CPPFLAGS

else
    echo "[!] Unsupported OS: $OS"
    exit 1
fi

echo "[+] Installing Rust toolchain..."
curl https://sh.rustup.rs -sSf | sh -s -- -y

# Activate rust in shell
source "$HOME/.cargo/env" || export PATH="$HOME/.cargo/bin:$PATH"

rustup toolchain install 1.88.0 --profile minimal
rustup default 1.88.0
rustup target add wasm32v1-none

echo "[✓] Environment setup complete."