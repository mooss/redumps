#!/usr/bin/env bash
set -euo pipefail

#############
# Functions #

function calc() {
    echo "$@" | bc -q
}

function ncores() { # Number of actual cores, not counting hyperthreading.
    grep 'core id' /proc/cpuinfo | sort -u | wc -l
}

######################
# Arguments handling #

if [[ $# -lt 2 ]]; then
    echo "Usage: $0 OUTPUT_DIR FILE [FILE...]" >&2
    exit 1
fi

redumps=./target/release/redumps
OUTPUT_DIR="$1"; shift
FILES=$(ls -lS "$@" | awk '{print $9}') # Sort files by size to always process the biggest first.

if [[ ! -x $redumps ]]; then
    echo -e "redumps binary ($redumps) not found.\nBuild it with \`just release\`." >&2
    exit 1
fi
start=$(date +%s%N)
total_bytes=0

#################
# Script proper #

printf '%s\n' "$FILES" | xargs -n1 -P $(ncores) $redumps count-fields -o "$OUTPUT_DIR" > /dev/null

for file in $FILES; do
    bytes=$(stat -c%s "$file")
    total_bytes=$((total_bytes + bytes))
done

end=$(date +%s%N)
elapsed_ns=$((end - start))
elapsed_s=$(echo "scale=9; $elapsed_ns / 1000000000" | bc)
mib=$(calc "scale=2; $total_bytes / 1048576")
throughput=$(echo "scale=2; $mib / $elapsed_s" | bc)

echo "$mib MiB processed in ${elapsed_s}s (throughput $throughput MiB/s)"
