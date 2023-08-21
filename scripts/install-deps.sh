#!/bin/bash

function install_macosx_darwin_22_deps()
{
    brew upgrade &&
    yes | brew install git cmake openssl protobuf
}

function install_deps()
{
    if [[ "$OSTYPE" == "linux-gnu" ]]; then
        sudo apt upgrade -y && \
        sudo apt install -y git build-essential make clang libssl-dev llvm libudev-dev protobuf-compiler
    elif [[ "$OSTYPE" == 'darwin22.0' ]]; then
        install_macosx_darwin_22_deps
    elif [[ "$OSTYPE" == 'darwin22' ]]; then
        install_macosx_darwin_22_deps
    elif [[ "$OSTYPE" == "darwin" ]]; then
        echo "NOT IMPLEMENTED: $OSTYPE"
        exit 1
    elif [[ "$OSTYPE" == "darwin20" ]]; then
        echo "NOT IMPLEMENTED: $OSTYPE"
        exit 1
    else
        echo "NOT IMPLEMENTED: $OSTYPE"
        exit 1
    fi
}

install_deps