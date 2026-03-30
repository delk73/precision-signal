#!/usr/bin/env python3

import argparse
import csv
import sys
from pathlib import Path


PREAMBLE_RUN = 10
PREAMBLE_VALUE = 200
PREAMBLE_TOLERANCE = 20


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("baseline")
    parser.add_argument("observed")
    return parser.parse_args()


def load_intervals(path: Path) -> list[int]:
    with path.open("r", newline="") as handle:
        reader = csv.reader(handle)
        header = next(reader, None)
        if header != ["index", "interval_us"]:
            raise SystemExit(f"{path}: invalid header")
        intervals = []
        expected_index = 0
        for row in reader:
            if len(row) != 2:
                raise SystemExit(f"{path}: invalid row")
            index = int(row[0])
            interval = int(row[1])
            if index != expected_index:
                raise SystemExit(f"{path}: non-contiguous index")
            intervals.append(interval)
            expected_index += 1
        return intervals


def find_alignment(intervals: list[int]) -> int:
    limit = len(intervals) - PREAMBLE_RUN + 1
    for i in range(limit):
        window = intervals[i : i + PREAMBLE_RUN]
        if all(abs(value - PREAMBLE_VALUE) <= PREAMBLE_TOLERANCE for value in window):
            return i + PREAMBLE_RUN
    raise SystemExit("alignment preamble not found")


def main() -> int:
    args = parse_args()
    baseline = load_intervals(Path(args.baseline))
    observed = load_intervals(Path(args.observed))

    baseline_start = find_alignment(baseline)
    observed_start = find_alignment(observed)

    payload_len = min(len(baseline) - baseline_start, len(observed) - observed_start)
    for frame in range(payload_len):
        baseline_value = baseline[baseline_start + frame]
        observed_value = observed[observed_start + frame]
        if baseline_value != observed_value:
            print("frame  baseline  observed")
            print(f"{frame:<5}  {baseline_value:<8}  {observed_value}   FIRST_DIVERGENCE")
            print()
            print(f"first_divergence_frame: {frame}")
            return 0

    raise SystemExit("no divergence found")


if __name__ == "__main__":
    sys.exit(main())
