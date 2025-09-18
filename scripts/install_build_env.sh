#!/bin/bash
set -e

echo "[*] Detecting platform..."
UNAME_OUT="$(uname -s)"
case "${UNAME_OUT}" in
    Linux*)     OS=Linux;;
    Darwin*)    OS=Mac;;
    *)          OS="UNKNOWN:${UNAME_OUT}"
esac

echo "[+] Platform: $OS"

if [ "$OS" = "Linux" ]; then
    echo "[+] Installing dependencies on Linux..."
    sed -i 's|http://archive.ubuntu.com/ubuntu|http://mirrors.edge.kernel.org/ubuntu|g' /etc/apt/sources.list || true
    apt-get update
    apt-get install -y curl build-essential protobuf-compiler clang git pkg-config libssl-dev llvm libudev-dev

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
