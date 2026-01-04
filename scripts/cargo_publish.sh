#!/usr/bin/env bash

set -e

# Get the script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "Publishing crate to crates.io..."
cd "$PROJECT_ROOT"

# Publish the crate
cargo publish

echo "Crate published successfully!"
echo ""
