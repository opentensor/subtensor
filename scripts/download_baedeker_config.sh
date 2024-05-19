#!/bin/bash

# Variables
REPO="opentensor/subtensor"
WORKFLOW_NAME="Baedeker"   
ARTIFACT_NAME="baedeker-config"
OUTPUT_DIR="/tmp/baedeker-config"

# Ensure GitHub CLI is installed
if ! command -v gh &> /dev/null
then
    echo "GitHub CLI (gh) could not be found. Please install it from https://cli.github.com/"
    exit 1
fi

# Get the latest workflow run ID for the specified workflow
WORKFLOW_RUN_ID=$(gh run list --repo "$REPO" --workflow="$WORKFLOW_NAME" --limit 1 --json databaseId -q '.[0].databaseId')

if [ -z "$WORKFLOW_RUN_ID" ]; then
    echo "No workflow runs found for $WORKFLOW_NAME"
    exit 1
fi

echo "Found workflow run ID: $WORKFLOW_RUN_ID"

# Download the artifact
gh run download "$WORKFLOW_RUN_ID" --name "$ARTIFACT_NAME" --dir "$OUTPUT_DIR"

if [ $? -ne 0 ]; then
    echo "Failed to download artifact $ARTIFACT_NAME"
    exit 1
fi

echo "Artifact $ARTIFACT_NAME downloaded to $OUTPUT_DIR"