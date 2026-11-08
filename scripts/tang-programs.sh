#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OFFSET=0x400000

if [[ $# -eq 0 ]]; then
    echo "uso: $0 <programa.txt>..." >&2
    exit 1
fi

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

python3 "$ROOT/tools/pseudoasm.py" "$@" -o "$work/programs.bin"
size="$(stat -c %s "$work/programs.bin")"

echo "Gravando $size bytes na flash da placa, no offset $OFFSET..."
openFPGALoader -b tangnano20k --external-flash -o "$OFFSET" --file-type bin --verify "$work/programs.bin"

openFPGALoader -b tangnano20k --external-flash --dump-flash -o "$OFFSET" --file-size "$size" "$work/readback.bin"
if cmp -s "$work/programs.bin" "$work/readback.bin"; then
    echo "Conferido: a flash tem a imagem nova."
else
    echo "A leitura de volta não bate com a imagem." >&2
    exit 1
fi
