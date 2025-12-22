#!/bin/bash

# Requires the samply crate to be installed:
# `cargo install samply`
# And this script should be run from the root of the project.
# Run with:
# `./scripts/gen_samply_flamegraph.sh`

CARGO_PROFILE_RELEASE_DEBUG=true samply record cargo run --release --bin zeckendorf --features plotting -- --deterministic
