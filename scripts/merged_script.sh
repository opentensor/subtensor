#!/bin/bash


usage() {
    echo "Usage: $0 -v <version>"
    exit 1
}

while getopts ":v:" opt; do
    case ${opt} in
        v)
            version=${OPTARG}
            ;;
        \?)
            usage
            ;;
    esac
done


if [ -z "$version" ]; then
    usage
fi

echo "[INFO] Starting the process for version $version"


echo "[INFO] Building the project with wasm-pack"

# Build command
wasm-pack build --release

# Wasm Blob file path
artifact_path="pkg/my_wasm_project_bg.wasm"


if [ ! -f "$artifact_path" ]; then
    echo "[ERROR] Artifact not found: $artifact_path"
    exit 1
fi

echo "[INFO] Artifact found at $artifact_path"

git checkout main

git fetch origin --tags

latest_tag=$(git describe --tags $(git rev-list --tags --max-count=1))

if [ -z "$latest_tag" ]; then
    echo "No tags found in the repository."
    exit 1
fi

commits=$(git log ${latest_tag}..main --pretty=format:"%H %ci %s" --reverse)

# List PRs merged since the last release
pr_list=()
while read -r hash date time timezone message; do
    if [[ $message =~ Merge\ pull\ request\ \#([0-9]+) ]]; then
        pr_number=${BASH_REMATCH[1]}
        pr_list+=("$pr_number")
    fi
done <<< "$commits"


if [ ${#pr_list[@]} -eq 0 ]; then
    echo "[ERROR] No PRs found since the last release"
    exit 1
fi

echo -e "[INFO] PRs found: ${pr_list[*]}"


get_pr_title() {
    local pr_number=$1
    gh pr view "$pr_number" --json title --jq '.title'
}


formatted_pr_list="PR numbers and their titles merged into main since the last tag ($latest_tag):\n\n"

for pr_number in "${pr_list[@]}"; do
    pr_title=$(get_pr_title "$pr_number")
    formatted_pr_list+="* PR #${pr_number} - ${pr_title}\n"
done


formatted_pr_list=$(printf "%b" "$formatted_pr_list")

echo -e "$formatted_pr_list"


echo "[INFO] Creating a new release with version $version and uploading the artifact"
if gh release create "$version" "$artifact_path" --title "$version" --notes "$formatted_pr_list"; then
    echo "[INFO] Release created successfully"
else
    echo "[ERROR] Failed to create the release"
    exit 1
fi

echo "[INFO] Release $version created and tagged successfully."
