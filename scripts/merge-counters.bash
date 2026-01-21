#!/usr/bin/env bash
# Merge JSON counters.
# Supports heterogenous counters.

set -euo pipefail

jq -s '
  reduce .[] as $o
    ({};
      reduce ($o | to_entries[]) as $e
        (.;
          .[$e.key] = (.[$e.key] // 0) + ($e.value // 0)
        )
    )
  | to_entries | sort_by(.value) | reverse | from_entries ' "$@"
