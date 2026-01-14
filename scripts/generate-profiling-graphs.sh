#!/bin/bash
set -euo pipefail

echo "Generating flamegraph..."
perf script | inferno-collapse-perf | inferno-flamegraph --width 2048 --height 32 --fontsize 12 > flamegraph.svg
echo "Flamegraph saved to flamegraph.svg"


echo "Generating callgraph..."
perf script | gprof2dot -f perf | dot -Tsvg -o callgraph.svg
echo "Callgraph SVG saved to callgraph.svg"

echo
echo "Note: the flamegraph is interactive, open it with a browser."
