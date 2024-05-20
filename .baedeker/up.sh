#!/bin/sh
set -e
BDK_DIR=$(dirname $(readlink -f "$0"))
RUST_LOG=info baedeker --spec=docker -J$BDK_DIR/vendor/ --generator=docker_compose=$BDK_DIR/.bdk-env --generator=docker_compose_discover=$BDK_DIR/.bdk-env/discover.env --secret=file=$BDK_DIR/.bdk-env/secret  --tla-str=relay_spec=rococo-local --input-modules='lib:baedeker-library/ops/nginx.libsonnet' --input-modules='lib:baedeker-library/ops/devtools.libsonnet' --tla-str=repoDir=$(realpath $BDK_DIR/..) $@ $BDK_DIR/rewrites.jsonnet
cd $BDK_DIR/.bdk-env

# Give Alice some balance
sed -i '/0x26aa394eea5630e07c48ae0c9558cef7b99d880ec681799c0cf30e8886371da9de1e86a9a8c739864cf3cc5ec2bea59fd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d/s/.*/0x00000000000000000100>

#docker compose up -d --wait --remove-orphans
