#!/bin/bash

function install_deps()
{
    if [[ "$OSTYPE" == "linux-gnu" ]]; then
        sudo apt upgrade -y && \
        sudo apt install -y git build-essential make clang libssl-dev llvm libudev-dev protobuf-compiler
    elif [[ "$OSTYPE" == 'darwin22.0' ]]; then
        brew upgrade &&
        yes | brew install git cmake openssl protobuf
    elif [[ "$OSTYPE" == 'darwin22' ]]; then
        brew upgrade &&
        yes | brew install git cmake openssl protobuf
        exit 1
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