#!/usr/bin/env bash
#
# Sobe o bode_os no QEMU.  Chamado pelo 'cargo run' (ver .cargo/config.toml),
# que passa o caminho do ELF como $1.
#
# Sair do QEMU:  Ctrl+A , solta , X
#
set -euo pipefail

qemu-system-riscv32 \
    -machine virt \
    -bios none \
    -nographic \
    -kernel "$1"
