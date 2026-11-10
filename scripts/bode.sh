#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ELF="$ROOT/target/riscv32imc-unknown-none-elf/release/bode_os"

usage() {
    cat >&2 <<HELP
uso: scripts/bode.sh <onde> [modo] [programas.txt...]

onde:  qemu | sim | placa
modo:  normal (padrao) | hw-test | bench | switch-bench | switch-bench-no-pmp

exemplos:
  scripts/bode.sh qemu
  scripts/bode.sh qemu normal programs/prog2.txt
  scripts/bode.sh sim hw-test
  scripts/bode.sh placa bench
  scripts/bode.sh placa normal programs/prog2.txt programs/contagem.txt
HELP
    exit 1
}

where="${1:-}"
mode="${2:-normal}"
shift $(( $# < 2 ? $# : 2 ))
programs=("$@")

case "$mode" in
    normal) feature="" ;;
    hw-test | bench | switch-bench | switch-bench-no-pmp) feature="$mode" ;;
    *) usage ;;
esac

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

assemble() {
    local sources=("${programs[@]}")
    if (( ${#sources[@]} == 0 )); then
        sources=("$ROOT/programs/prog1.txt" "$ROOT/programs/contagem.txt")
    fi
    python3 "$ROOT/tools/pseudoasm.py" "${sources[@]}" -o "$work/programs.bin" >/dev/null
    echo "$work/programs.bin"
}

cd "$ROOT"

case "$where" in
    qemu)
        BODE_FLASH="$(assemble)" cargo run ${feature:+--features "$feature"}
        ;;
    sim)
        cargo build --release --features "ibex${feature:+,$feature}"
        BODE_FLASH="$(assemble)" scripts/run-ibex.sh "$ELF"
        ;;
    placa)
        if (( ${#programs[@]} > 0 )); then
            scripts/nexys-programs.sh "${programs[@]}"
        fi
        cargo build --release --features "ibex${feature:+,$feature}"
        if [[ -n "${NEXYS_FULL:-}" ]]; then
            scripts/nexys-flash.sh "$(scripts/nexys-build.sh "$(scripts/vmem.sh)" "$mode")"
        else
            scripts/nexys-flash.sh "$(scripts/nexys-base.sh)" "$ELF"
        fi
        ;;
    *)
        usage
        ;;
esac
