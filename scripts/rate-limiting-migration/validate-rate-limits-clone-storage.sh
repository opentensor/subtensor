#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

cd "$REPO_ROOT/ts-tests"
export NODE_PATH="$PWD/node_modules"
export NODE_OPTIONS="${NODE_OPTIONS:+$NODE_OPTIONS }--max-old-space-size=8192"
pnpm exec tsx ../scripts/rate-limiting-migration/validate-rate-limits-clone-storage.ts "$@"
