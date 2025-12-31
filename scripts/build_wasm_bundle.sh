#!/usr/bin/env bash

which wasm-pack > /dev/null || {
    echo "wasm-pack is not installed. Please install it using the following command:"
    echo "cargo install wasm-pack"
    exit 1
}

# Outputs the bundle to the pkg directory.

wasm-pack build
