#!/usr/bin/env bash
#
# Sobe o bode_os no QEMU.  Chamado pelo 'cargo run' (ver .cargo/config.toml),
# que passa o caminho do ELF como $1.
#
#   cargo run           roda normal
#   cargo run -- -g     congela no reset e espera o GDB na porta 1234
#
#   BODE_FLASH=programas.bin cargo run    põe a imagem na flash, no offset dos programas
#
# Sair do QEMU:  Ctrl+A , solta , X
#
set -euo pipefail

KERNEL=""
GDB=()

for arg in "$@"; do
    case "$arg" in
        -g|--gdb) GDB=(-s -S) ;;
        -*)       echo "run.sh: flag desconhecida '$arg'" >&2; exit 1 ;;
        *)        KERNEL="$arg" ;;
    esac
done

if (( ${#GDB[@]} )); then
    echo ">> QEMU congelado na porta 1234. Noutro terminal:"
    echo "   gdb -ex 'set architecture riscv:rv32' -ex 'target remote :1234' $KERNEL"
    echo
fi

ROOT="$(cd "$(dirname "$0")/.." && pwd)"

FLASH=()
if [[ "${BODE_FLASH:-}" != none ]]; then
    work="$(mktemp -d)"
    trap 'rm -rf "$work"' EXIT
    image="${BODE_FLASH:-$work/programs.bin}"
    if [[ -z "${BODE_FLASH:-}" ]]; then
        python3 "$ROOT/tools/pseudoasm.py" "$ROOT/programs/prog1.txt" "$ROOT/programs/contagem.txt" -o "$image" >/dev/null
    fi
    truncate -s 32M "$work/pflash"
    dd if="$image" of="$work/pflash" bs=4096 seek=$((0x400000)) oflag=seek_bytes conv=notrunc status=none
    FLASH=(-drive "if=pflash,unit=1,format=raw,file=$work/pflash")
fi

qemu-system-riscv32 \
    -machine virt \
    -bios none \
    -m 128M \
    -nographic \
    "${GDB[@]}" \
    "${FLASH[@]}" \
    -kernel "$KERNEL"
