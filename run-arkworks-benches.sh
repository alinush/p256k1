#!/usr/bin/env bash
#
# Benchmarks ark-secp256k1's multi-scalar multiplication (MSM) against a naive
# loop of single scalar multiplications, for n = 2, 4, 8, ..., 1024.
#
# Outputs a Markdown table with mean times (in microseconds) parsed from
# Criterion's per-benchmark estimates.json.
#
# Requires: cargo, jq, awk, bc.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CRATE_DIR="$SCRIPT_DIR/p256k1"
# In a Cargo workspace, target/ lives at the workspace root.
TARGET_DIR="$(cd "$SCRIPT_DIR" && cargo metadata --format-version 1 --no-deps 2>/dev/null \
    | jq -r '.target_directory' 2>/dev/null || echo "$SCRIPT_DIR/target")"

for tool in cargo jq awk bc; do
    if ! command -v "$tool" >/dev/null 2>&1; then
        echo "Missing required tool: $tool" >&2
        exit 1
    fi
done

echo "Running Criterion benchmarks (this takes ~1-2 minutes)..." >&2
(cd "$CRATE_DIR" && cargo bench --bench arkworks_msm_bench -- --quiet >/dev/null)

CRIT_DIR="$TARGET_DIR/criterion/ark_msm"

mean_us() {
    local f="$1"
    if [[ -f "$f" ]]; then
        # estimates.json reports point estimates in nanoseconds
        jq -r '.mean.point_estimate' "$f" \
            | awk '{ printf "%.3f", $1 / 1000.0 }'
    else
        echo "NA"
    fi
}

echo
echo "| n    | MSM (µs)   | Naive (µs) | Speedup | MSM µs / n  | Naive µs / n |"
echo "|-----:|-----------:|-----------:|--------:|------------:|-------------:|"
for n in 2 4 8 16 32 64 128 256 512 1024; do
    msm_f="$CRIT_DIR/multimult/$n/new/estimates.json"
    naive_f="$CRIT_DIR/naive/$n/new/estimates.json"
    msm_us=$(mean_us "$msm_f")
    naive_us=$(mean_us "$naive_f")
    if [[ "$msm_us" != "NA" && "$naive_us" != "NA" ]]; then
        speedup=$(echo "scale=2; $naive_us / $msm_us" | bc -l)
        msm_per_n=$(echo "scale=3; $msm_us / $n" | bc -l)
        naive_per_n=$(echo "scale=3; $naive_us / $n" | bc -l)
        printf "| %4d | %10s | %10s | %6sx | %11s | %12s |\n" \
            "$n" "$msm_us" "$naive_us" "$speedup" "$msm_per_n" "$naive_per_n"
    else
        printf "| %4d | %10s | %10s |       - |           - |            - |\n" \
            "$n" "$msm_us" "$naive_us"
    fi
done
