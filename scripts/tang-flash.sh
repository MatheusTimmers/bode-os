#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
IBEX="${IBEX_DEMO_SYSTEM:-$ROOT/../ibex-demo-system}"
BITSTREAM="${1:-$IBEX/build/tangnano20k/top_tangnano20k.fs}"
UART="${TANG_UART:-/dev/ttyUSB1}"

if [[ ! -f "$BITSTREAM" ]]; then
    echo "bitstream não encontrado: $BITSTREAM" >&2
    exit 1
fi

read -rp "Reconecte o cabo USB da placa e tecle Enter... "

if [[ ! -e "$UART" ]]; then
    echo "UART não encontrada: $UART" >&2
    exit 1
fi

log="$(mktemp)"
stty -F "$UART" 115200 raw -echo -crtscts
cat "$UART" &
reader=$!
trap 'kill "$reader" 2>/dev/null; rm -f "$log"' EXIT
trap 'exit 130' INT TERM

echo "Gravando $BITSTREAM..."
if ! openFPGALoader -b tangnano20k "$BITSTREAM" > "$log" 2>&1; then
    cat "$log" >&2
    exit 1
fi
echo "Gravado. Saída da UART abaixo (Ctrl+C para sair):"

wait "$reader"
