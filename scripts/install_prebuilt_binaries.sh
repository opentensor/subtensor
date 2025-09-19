#!/bin/bash

# We move the prebuild binaries required by the architecture if they were created in CI, otherwise exit with no error
if [ -z "$BUILT_IN_CI" ]; then
  echo "[*] BUILT_IN_CI is not set. Skipping script..."
  exit 0
fi

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

set -x

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
  echo "[*] Listing files in /build/ci_target/${RUNTIME}/${BUILD_TRIPLE}/release/"
  ls -al /build/ci_target/${RUNTIME}/${BUILD_TRIPLE}/release/ || true
  echo "[*] Listing wasm in wbuild/"
  ls -al /build/ci_target/${RUNTIME}/${BUILD_TRIPLE}/release/wbuild/node-subtensor-runtime/ || true

  mkdir -p /build/target/${RUNTIME}/release/wbuild/node-subtensor-runtime
  cp -v /build/ci_target/${RUNTIME}/${BUILD_TRIPLE}/release/node-subtensor \
        /build/target/${RUNTIME}/release/node-subtensor
  cp -v /build/ci_target/${RUNTIME}/${BUILD_TRIPLE}/release/wbuild/node-subtensor-runtime/node_subtensor_runtime.compact.compressed.wasm \
        /build/target/${RUNTIME}/release/wbuild/node-subtensor-runtime/node_subtensor_runtime.compact.compressed.wasm
done