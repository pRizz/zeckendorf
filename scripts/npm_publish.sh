#!/usr/bin/env bash

set -e

# Get the script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "Building WASM bundle..."
cd "$PROJECT_ROOT"

# Build the WASM bundle
"$SCRIPT_DIR/build_wasm_bundle.sh"

echo "Publishing to npm..."
cd "$PROJECT_ROOT/pkg"

# Publish the WASM package
wasm-pack publish

echo "Package published successfully!"
