#!/usr/bin/env bash
set -euo pipefail

# The purpose of this script is to poll the RSS (resident set size (essentially memory usage)) in kilobytes of the zeckendorf program and print the peak RSS.
# Run with:
# `./scripts/poll_rss.sh`
#
# I am currently running into a problem with decompression taking an inordinate amount of memory.
# For example, decompressing 10,000 bytes of all ones data takes ~1.87 GB of memory, and
# attempting to decompress 100,000 bytes of all ones data takes >60 GB of memory and causes the process to be killed by the OS (exit code 137). This is likely due to the Fibonacci memoization cache growing too large at high indices. Perhaps we can limit the size of the cache or do a more sparse cache using the fast doubling Fibonacci algorithm.
# Snapshot on 2025-12-27 with a limit of 10,000 bytes of all ones data:
# Peak RSS: 1962032 KB (1916.05 MB, 1.87 GB)
# Snapshot on 2025-12-27 with a limit of 20,000 bytes of all ones data:
# Peak RSS: 4962928 KB (4846.61 MB, 4.73 GB)
# Snapshot on 2025-12-27 with a limit of 40,000 bytes of all ones data:
# Peak RSS: 20016880 KB (19547.73 MB, 19.09 GB)
# So decompressing 20,000 bytes was 2.5x more memory intensive than decompressing 10,000 bytes and
# decompressing 40,000 bytes was 10.2x more memory intensive than decompressing 10,000 bytes.
# The memory usage is growing non-linearly with the size of the input data.
# After testing out using the fast doubling Fibonacci algorithm, the memory usage is now much better, at
# Snapshot on 2025-12-27 with a limit of 10,000 bytes of all ones data using the fast doubling Fibonacci algorithm:
# Peak RSS: 43056 KB (42.05 MB, 0.04 GB)
# Snapshot on 2025-12-27 with a limit of 20,000 bytes of all ones data using the fast doubling Fibonacci algorithm:
# Peak RSS: 43104 KB (42.09 MB, 0.04 GB)
# Snapshot on 2025-12-27 with a limit of 40,000 bytes of all ones data using the fast doubling Fibonacci algorithm:
# Peak RSS: 43344 KB (42.33 MB, 0.04 GB)
# But, the time taken to decompress is now much slower, at
# Time taken to test all ones decompressions with 10,000 bytes of all ones data: 21s
# Time taken to test all ones decompressions with 20,000 bytes of all ones data: 124s
# Time taken to test all ones decompressions with 40,000 bytes of all ones data: 754s or 12.5 minutes
# So, the fast doubling Fibonacci algorithm is not a silver bullet.
# TODO: investigate ways we can get the lower memory usage of the cached fast doubling Fibonacci algorithm but the speed of the cached slow Fibonacci algorithm. As of now, the cached fast doubling Fibonacci algorithm is slower at decompression than the cached slow Fibonacci algorithm at large data inputs, on the order of > 10kB.

cargo run --release --bin zeckendorf --features plotting -- --deterministic &
pid=$!

peak=0
while kill -0 "$pid" 2>/dev/null; do
  rss_kb=$(ps -o rss= -p "$pid" | tr -d ' ')
  rss_kb=${rss_kb:-0}
  (( rss_kb > peak )) && peak=$rss_kb
  sleep 0.02
done

wait "$pid"
peak_mb=$(awk "BEGIN {printf \"%.2f\", $peak/1024}")
peak_gb=$(awk "BEGIN {printf \"%.2f\", $peak/1048576}")
echo "Peak RSS: $peak KB ($peak_mb MB, $peak_gb GB)"
