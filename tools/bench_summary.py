#!/usr/bin/env python3
import csv
import sys
from collections import defaultdict
from pathlib import Path

FIELDS = [
    "alloc_n", "alloc_min", "alloc_media", "alloc_max", "alloc_total",
    "free_n", "free_min", "free_media", "free_max", "free_total",
    "falhas", "falha_media", "falha_max", "pico_pedido", "pico_consumido",
    "recusas", "falhas_fragmentacao",
]
BIN_WIDTH = 32


def percentile(bins, fraction):
    total = sum(bins)
    if total == 0:
        return 0
    seen = 0
    for index, count in enumerate(bins):
        seen += count
        if seen >= total * fraction:
            return (index + 1) * BIN_WIDTH
    return len(bins) * BIN_WIDTH


def add_bins(target, bins):
    if not target:
        target.extend([0] * len(bins))
    for index, count in enumerate(bins):
        target[index] += count


def main():
    runs = defaultdict(list)
    histograms = defaultdict(list)
    pmp = None

    with open(Path(sys.argv[1]), newline="") as file:
        for row in csv.reader(file):
            if row[:2] == ["pmp", "allow_region"]:
                pmp = row[2:]
            elif len(row) == 21 and row[0] == "bench" and row[3] != "semente":
                runs[row[1], row[2]].append(dict(zip(FIELDS, map(int, row[4:]))))
            elif row and row[0] == "hist":
                add_bins(histograms[row[1], row[2], row[4]], list(map(int, row[5:])))

    print("| cenario | alocador | alloc min | alloc media | alloc p50 | alloc p99 | alloc pior | free media | free pior | falhas | pior falha | pico pedido | pico consumido | recusas | falhas por fragmentacao |")
    print("|" + " --- |" * 15)
    for key, group in runs.items():
        scenario, allocator = key
        alloc_n = sum(run["alloc_n"] for run in group)
        free_n = sum(run["free_n"] for run in group)
        bins = histograms[scenario, allocator, "alloc"]
        cells = [
            scenario,
            allocator,
            min((run["alloc_min"] for run in group if run["alloc_n"]), default=0),
            f"{sum(run['alloc_total'] for run in group) / max(alloc_n, 1):.1f}",
            f"<={percentile(bins, 0.50)}",
            f"<={percentile(bins, 0.99)}",
            max(run["alloc_max"] for run in group),
            f"{sum(run['free_total'] for run in group) / max(free_n, 1):.1f}",
            max(run["free_max"] for run in group),
            sum(run["falhas"] for run in group),
            max(run["falha_max"] for run in group),
            max(run["pico_pedido"] for run in group),
            max(run["pico_consumido"] for run in group),
            sum(run["recusas"] for run in group),
            "-" if allocator == "bump" else sum(run["falhas_fragmentacao"] for run in group),
        ]
        print("| " + " | ".join(map(str, cells)) + " |")

    if pmp:
        print(f"\nallow_region: {pmp[0]} amostras, min {pmp[1]}, media {pmp[2]}, pior {pmp[3]} ciclos")


if __name__ == "__main__":
    main()
