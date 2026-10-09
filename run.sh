#!/usr/bin/env bash
#
# Sobe o bode_os no QEMU.  Chamado pelo 'cargo run' (ver .cargo/config.toml),
# que passa o caminho do ELF como $1.
#
#   cargo run           roda normal
#   cargo run -- -g     congela no reset e espera o GDB na porta 1234
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

qemu-system-riscv32 \
    -machine virt \
    -bios none \
    -m 128M \
    -nographic \
    "${GDB[@]}" \
    -kernel "$KERNEL"
