#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
IBEX="${IBEX_DEMO_SYSTEM:-$ROOT/../ibex-demo-system}"
BASE="$IBEX/build/nexysa7/base/base.bit"

if [[ -f "$BASE" && -z "$(find "$IBEX/rtl" "$IBEX/data" "$IBEX/ibex_demo_system.core" -newer "$BASE" -print -quit)" ]]; then
    echo "$BASE"
    exit 0
fi

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
echo "@00000020 0000006F" > "$work/idle.vmem"

"$ROOT/scripts/nexys-build.sh" "$work/idle.vmem" base
