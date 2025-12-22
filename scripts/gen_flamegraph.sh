#!/bin/bash

# Requires the flamegraph crate to be installed:
# `cargo install flamegraph`
# And this script should be run from the root of the project, as root.
# Run with:
# `sudo ./scripts/gen_flamegraph.sh`
# Root permissions are required to properly profile the program.

CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph --root --bin zeckendorf --features plotting -- --deterministic
