#!/bin/sh

set -e  # Exit immediately if a command exits with a non-zero status.

# Function to check for git changes and commit if necessary.
commit_if_changes() {
    if [ -n "$(git status --porcelain)" ]; then
        echo "changes detected, committing..."
        git commit -am "$1"
        echo "commit created."
    fi
}

# Step 1: Run cargo check and commit changes to Cargo.lock if any
cargo check --workspace
commit_if_changes "commit Cargo.lock"

# Step 2: Run cargo clippy with fixes and commit changes if any.
cargo clippy --fix --workspace --all-features --all-targets
commit_if_changes "cargo clippy"

# Step 3: Run cargo fix and commit changes if any.
cargo fix --workspace --all-features --all-targets
commit_if_changes "cargo fix"

# Step 4: Run cargo fmt and commit changes if any.
cargo fmt --all
commit_if_changes "cargo fmt"

if command -v zepter >/dev/null 2>&1; then
    echo "zepter detected, running 'zepter run check'..."
    zepter run check
fi