#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd)"
SIM="${IBEX_SIM:-$ROOT/../ibex-demo-system/build/lowrisc_ibex_demo_system_0/sim-verilator/Vtop_verilator}"
KERNEL="$1"

if [[ ! -x "$SIM" ]]; then
    echo "run-ibex.sh: simulador nao encontrado em $SIM" >&2
    echo "  compile no ibex-demo-system com:" >&2
    echo "  fusesoc --cores-root=. run --target=sim --tool=verilator --setup --build lowrisc:ibex:demo_system" >&2
    exit 1
fi

"$ROOT/scripts/vmem.sh" "$KERNEL" "$KERNEL.vmem" >/dev/null

exec "$SIM" --meminit=ram,"$KERNEL.vmem" +UARTDPI_LOG_uart0=-
