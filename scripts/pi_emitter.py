#!/usr/bin/env python3

import argparse
import sys
import time

try:
    import gpiod
    from gpiod.line import Direction, Value
except ImportError as exc:
    raise SystemExit(
        f"gpiod import failed: {exc}. Install the Python gpiod bindings "
        "(for example: python3-libgpiod)."
    )


GPIO = 17
GPIO_CHIP = "/dev/gpiochip0"
CONSUMER = "precision-signal-pi-emitter"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--mode", required=True, choices=("baseline", "perturb"))
    parser.add_argument("--frames", type=int, default=128)
    parser.add_argument("--perturb-frame", type=int, default=50)
    parser.add_argument("--preamble-count", type=int, default=10)
    parser.add_argument("--repeat-seconds", type=float)
    return parser.parse_args()


def build_intervals(args: argparse.Namespace) -> list[int]:
    intervals = [200] * args.preamble_count
    payload = [1000] * args.frames
    if args.mode == "perturb":
        if not 0 <= args.perturb_frame < args.frames:
            raise SystemExit("--perturb-frame out of range for --frames")
        payload[args.perturb_frame] = 1500
    intervals.extend(payload)
    return intervals


def request_output_line():
    try:
        return gpiod.request_lines(
            GPIO_CHIP,
            consumer=CONSUMER,
            config={
                GPIO: gpiod.LineSettings(
                    direction=Direction.OUTPUT,
                    output_value=Value.INACTIVE,
                )
            },
        )
    except OSError as exc:
        raise SystemExit(
            f"failed to request GPIO{GPIO} on {GPIO_CHIP}: {exc}. "
            "Check GPIO character device access."
        ) from exc


def wait_until(deadline_ns: int) -> None:
    while True:
        remaining_ns = deadline_ns - time.perf_counter_ns()
        if remaining_ns <= 0:
            return
        if remaining_ns > 50_000:
            time.sleep((remaining_ns - 25_000) / 1_000_000_000)


def emit_intervals(request, intervals: list[int]) -> None:
    deadline_ns = time.perf_counter_ns()
    request.set_value(GPIO, Value.INACTIVE)
    for interval in intervals:
        if interval % 2 != 0:
            raise SystemExit(f"interval must be even: {interval}")

        half_ns = (interval // 2) * 1_000

        deadline_ns += half_ns
        request.set_value(GPIO, Value.ACTIVE)
        wait_until(deadline_ns)

        deadline_ns += half_ns
        request.set_value(GPIO, Value.INACTIVE)
        wait_until(deadline_ns)

    request.set_value(GPIO, Value.INACTIVE)


def emit_for_duration(
    request, intervals: list[int], repeat_seconds: float | None
) -> None:
    if repeat_seconds is None:
        emit_intervals(request, intervals)
        return
    if repeat_seconds <= 0:
        raise SystemExit("--repeat-seconds must be > 0")

    end_time = time.perf_counter() + repeat_seconds
    while True:
        emit_intervals(request, intervals)
        if time.perf_counter() >= end_time:
            request.set_value(GPIO, Value.INACTIVE)
            return


def main() -> int:
    args = parse_args()
    intervals = build_intervals(args)
    request = request_output_line()
    try:
        emit_for_duration(request, intervals, args.repeat_seconds)
        return 0
    finally:
        request.set_value(GPIO, Value.INACTIVE)
        request.release()


if __name__ == "__main__":
    sys.exit(main())
