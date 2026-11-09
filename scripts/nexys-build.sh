#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
IBEX="${IBEX_DEMO_SYSTEM:-$ROOT/../ibex-demo-system}"
HOST="${NEXYS_HOST:-paxos-ext}"
SSH_OPTIONS=(-o BatchMode=yes -o LogLevel=ERROR)

if [[ $# -ne 2 ]]; then
    echo "uso: $0 <kernel.vmem> <nome>" >&2
    exit 1
fi

vmem="$1"
name="$2"
build="$IBEX/build/nexysa7/$name"
project="synth_nexysa7-vivado"
remote_home="$(ssh "${SSH_OPTIONS[@]}" "$HOST" 'echo $HOME' 2>/dev/null)"
remote="bode_nexys/$name"

rm -rf "$build"
mkdir -p "$build"
cp "$vmem" "$build/kernel.vmem"

cd "$IBEX"
PATH="$IBEX/.venv/bin:$PATH" fusesoc --cores-root=. run --build-root="$build" \
    --target=synth_nexysa7 --setup lowrisc:ibex:demo_system \
    --SRAMInitFile="$remote_home/$remote/kernel.vmem" >&2

ssh "${SSH_OPTIONS[@]}" "$HOST" "mkdir -p $remote" 2>/dev/null
rsync -az --delete -e "ssh ${SSH_OPTIONS[*]}" "$build/" "$HOST:$remote/" 2>/dev/null

echo "Sintetizando no $HOST..." >&2
ssh "${SSH_OPTIONS[@]}" "$HOST" \
    "bash -lc 'module load vivado/2025.2 && cd $remote/$project && make'" >&2

report="$(ssh "${SSH_OPTIONS[@]}" "$HOST" \
    "bash -c 'grep -l \"All user specified timing constraints are met\" $remote/$project/*.runs/impl_1/*timing_summary_routed.rpt'" 2>/dev/null || true)"
if [[ -z "$report" ]]; then
    echo "o circuito de $name nao fechou o tempo" >&2
    exit 1
fi

bitstream="$build/$name.bit"
scp -q "${SSH_OPTIONS[@]}" "$HOST:$remote/$project/lowrisc_ibex_demo_system_0.bit" "$bitstream" 2>/dev/null
echo "$bitstream"
