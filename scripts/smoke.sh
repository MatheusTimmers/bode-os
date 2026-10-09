#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
KERNEL="${1:-$ROOT/target/riscv32imac-unknown-none-elf/debug/bode_os}"
DURATION="${SMOKE_SECONDS:-3}"

output="$(timeout "$DURATION" "$ROOT/run.sh" "$KERNEL" </dev/null 2>&1 | tr -d '\r' || true)"
echo "$output"
echo "----"

status=0
for expected in "Iniciando o Kernel BODE..." "A" "B"; do
    if ! grep -qxF "$expected" <<<"$output"; then
        echo "smoke: faltou a linha '$expected'" >&2
        status=1
    fi
done

for forbidden in "encerrada" "Exception no kernel" "panicked"; do
    if grep -qF "$forbidden" <<<"$output"; then
        echo "smoke: apareceu '$forbidden'" >&2
        status=1
    fi
done

if (( status == 0 )); then
    echo "smoke: ok"
fi
exit "$status"
