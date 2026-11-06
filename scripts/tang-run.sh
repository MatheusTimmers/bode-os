#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
IBEX="${IBEX_DEMO_SYSTEM:-$ROOT/../ibex-demo-system}"

cd "$ROOT"
cargo build-ibex
vmem="$(scripts/vmem.sh)"

echo "Sintetizando o bitstream (cerca de 13 min)..."
bitstream="$("$IBEX/util/synth_tangnano20k.sh" "$vmem" | tail -n 1)"

exec "$ROOT/scripts/tang-flash.sh" "$bitstream"
