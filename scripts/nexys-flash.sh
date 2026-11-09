#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
IBEX="${IBEX_DEMO_SYSTEM:-$ROOT/../ibex-demo-system}"
UART="${NEXYS_UART:-$(echo /dev/serial/by-id/usb-Digilent_*-if01-port0)}"

if [[ $# -lt 1 || $# -gt 2 ]]; then
    echo "uso: $0 <bitstream.bit> [kernel.elf]" >&2
    exit 1
fi

stty -F "$UART" 115200 raw -echo -crtscts
cat "$UART" &
reader=$!
trap 'kill "$reader" 2>/dev/null' EXIT
trap 'exit 130' INT TERM

openFPGALoader -b nexys_a7_100 "$1" >&2

if [[ $# -eq 2 ]]; then
    "$IBEX/util/load_demo_system.sh" run "$2" "$IBEX/util/nexysa7-openocd-cfg.tcl" >/dev/null 2>&1
fi

echo "Gravado. Saída da UART abaixo (Ctrl+C para sair):" >&2

wait "$reader"
