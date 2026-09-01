#!/usr/bin/env bash
# Compile the deliberately-broken examples one at a time and show the error.
#
#   ./show-errors.sh            # all of them, pausing between each
#   ./show-errors.sh e0499      # just the one you want on screen
#
# Nothing here is part of `cargo build` — these files live outside src/ on
# purpose, so the package always compiles.
set -u
cd "$(dirname "$0")/broken" || exit 1

filter="${1:-}"
for f in *.rs; do
    [ -n "$filter" ] && [[ "$f" != *"$filter"* ]] && continue
    echo
    echo "════════════════════════════════════════════════════════════"
    echo "  $f"
    echo "════════════════════════════════════════════════════════════"
    sed -n '1,12p' "$f" | grep '^//' | sed 's|^// \{0,1\}|  |'
    echo "  ── the code ──"
    grep -v '^//' "$f" | sed '/^$/d' | sed 's/^/  /'
    echo "  ── rustc says ──"
    rustc --edition 2021 --emit=metadata -o /dev/null "$f" 2>&1 | sed 's/^/  /'
    if [ -z "$filter" ]; then
        read -r -p "  [enter for the next one] " _ < /dev/tty || exit 0
    fi
done
