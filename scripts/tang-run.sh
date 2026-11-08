#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
IBEX="${IBEX_DEMO_SYSTEM:-$ROOT/../ibex-demo-system}"
PNR="$IBEX/build/tangnano20k/top_tangnano20k.pnr.json"
FEATURES="ibex${BODE_FEATURES:+,$BODE_FEATURES}"

full=false
if [[ "${1:-}" == "--full" ]]; then
    full=true
fi
if [[ ! -f "$PNR" ]]; then
    full=true
elif [[ -n "$(find "$IBEX/rtl" "$IBEX/data" "$IBEX/ibex_demo_system.core" -newer "$PNR" -print -quit)" ]]; then
    full=true
fi

cd "$ROOT"
cargo build --release --features "$FEATURES"
vmem="$(scripts/vmem.sh)"

if [[ "$full" == true ]]; then
    echo "Sintetizando o circuito completo (cerca de 11 min)..."
    bitstream="$("$IBEX/util/synth_tangnano20k.sh" "$vmem" | tail -n 1)"
else
    echo "Hardware igual: trocando so o kernel no bitstream..."
    bitstream="$("$IBEX/util/update_tangnano20k_kernel.sh" "$vmem" | tail -n 1)"
fi

exec "$ROOT/scripts/tang-flash.sh" "$bitstream"
