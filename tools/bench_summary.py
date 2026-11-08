#!/usr/bin/env python3
import csv
import sys
from collections import defaultdict
from pathlib import Path

FIELDS = ["alloc_n", "alloc_min", "alloc_media", "alloc_max", "free_n", "free_min", "free_media", "free_max", "falhas", "pico_bytes"]


def rows(path):
    with open(path, newline="") as file:
        for row in csv.reader(file):
            if len(row) == 14 and row[0] == "bench" and row[3] != "semente":
                yield row[1], row[2], dict(zip(FIELDS, map(int, row[4:])))


def main():
    groups = defaultdict(list)
    for scenario, allocator, values in rows(Path(sys.argv[1])):
        groups[scenario, allocator].append(values)

    header = "| cenario | alocador | alloc media | alloc pior | free media | free pior | falhas | pico (bytes) | sementes |"
    print(header)
    print("|" + " --- |" * 9)
    for (scenario, allocator), runs in groups.items():
        total_alloc = sum(run["alloc_n"] for run in runs)
        total_free = sum(run["free_n"] for run in runs)
        alloc_mean = sum(run["alloc_media"] * run["alloc_n"] for run in runs) / max(total_alloc, 1)
        free_mean = sum(run["free_media"] * run["free_n"] for run in runs) / max(total_free, 1)
        print(
            f"| {scenario} | {allocator} | {alloc_mean:.0f} | {max(run['alloc_max'] for run in runs)} "
            f"| {free_mean:.0f} | {max(run['free_max'] for run in runs)} "
            f"| {sum(run['falhas'] for run in runs)} | {max(run['pico_bytes'] for run in runs)} | {len(runs)} |"
        )


if __name__ == "__main__":
    main()
