#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
KERNEL="${1:-$ROOT/target/riscv32imc-unknown-none-elf/debug/bode_os}"
DURATION="${SMOKE_SECONDS:-3}"

output="$(timeout "$DURATION" "$ROOT/run.sh" "$KERNEL" </dev/null 2>&1 | tr -d '\r' || true)"
echo "$output"
echo "----"

status=0
for expected in "flash: nenhuma imagem de programas" "Iniciando o Kernel BODE..."; do
    if ! grep -qxF "$expected" <<<"$output"; then
        echo "smoke: faltou a linha '$expected'" >&2
        status=1
    fi
done

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
python3 "$ROOT/tools/t1asm.py" "$ROOT/programs/prog1.txt" "$ROOT/programs/contagem.txt" -o "$work/programs.bin" >/dev/null
flash_output="$(BODE_FLASH="$work/programs.bin" timeout "$DURATION" "$ROOT/run.sh" "$KERNEL" </dev/null 2>&1 | tr -d '\r' || true)"
echo "$flash_output"
echo "----"

for expected in "P0: fim, acc = 11" "P1: 9" "P1: fim, acc = 0"; do
    if ! grep -qxF "$expected" <<<"$flash_output"; then
        echo "smoke: com a flash, faltou a linha '$expected'" >&2
        status=1
    fi
done

for forbidden in "encerrada" "Exception no kernel" "panicked"; do
    if grep -qF "$forbidden" <<<"$output$flash_output"; then
        echo "smoke: apareceu '$forbidden'" >&2
        status=1
    fi
done

if (( status == 0 )); then
    echo "smoke: ok"
fi
exit "$status"
