#!/usr/bin/env bash

set -e

# Get the script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "Publishing crate to crates.io and npm..."
cd "$PROJECT_ROOT"

# Publish the crate to crates.io
"$SCRIPT_DIR/cargo_publish.sh"

# Publish the WASM package to npm
"$SCRIPT_DIR/npm_publish.sh"

# Call the git tag script to tag the release
"$SCRIPT_DIR/git_tag_current_version.sh"
