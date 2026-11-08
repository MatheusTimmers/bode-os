#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
KERNEL="${1:-$ROOT/target/riscv32imc-unknown-none-elf/release/bode_os}"
OUT="${2:-$KERNEL.vmem}"
RAM_START=0x00100000

bin="$(mktemp)"
trap 'rm -f "$bin"' EXIT

llvm-objcopy -O binary "$KERNEL" "$bin"
offset=$(( $(llvm-nm "$KERNEL" | awk '$3 == "__text_start" { print "0x" $1 }') - RAM_START ))
srec_cat "$bin" -binary -offset "$offset" -fill 0x00 -within "$bin" -binary -offset "$offset" -range-padding 4 -byte-swap 4 -o "$OUT" -vmem

echo "$OUT"
