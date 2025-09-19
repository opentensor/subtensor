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

echo "[*] BUILT_IN_CI is set → using prebuilt binaries."
echo "[*] Mapping TARGETARCH=${TARGETARCH} to Rust triple..."

# BUILD_TRIPLE are located on `.github/workflows/docker-localnet.yml` in a job `build:matrix:platform:triple`
# If these are updated in the workflow, then we need to update here in `elif [ -d "/build/ci_target" ]` section.
# We substitute the related binaries for the required Docker image layer architecture.
if [ "$TARGETARCH" = "amd64" ]; then
  BUILD_TRIPLE="x86_64-unknown-linux-gnu"
elif [ "$TARGETARCH" = "arm64" ]; then
  BUILD_TRIPLE="aarch64-unknown-linux-gnu"
else
  echo "[!] Unknown TARGETARCH: ${TARGETARCH}" >&2
  exit 1
fi

echo "[*] Using BUILD_TRIPLE=$BUILD_TRIPLE"
echo "[*] Copying binaries to expected /build/target layout..."

for RUNTIME in fast-runtime non-fast-runtime; do
  mkdir -p /build/target/${RUNTIME}/release/wbuild/node-subtensor-runtime
  cp -v /build/ci_target/${RUNTIME}/${BUILD_TRIPLE}/release/node-subtensor \
        /build/target/${RUNTIME}/release/node-subtensor
  cp -v /build/ci_target/${RUNTIME}/${BUILD_TRIPLE}/release/wbuild/node-subtensor-runtime/node_subtensor_runtime.compact.compressed.wasm \
        /build/target/${RUNTIME}/release/wbuild/node-subtensor-runtime/node_subtensor_runtime.compact.compressed.wasm
done