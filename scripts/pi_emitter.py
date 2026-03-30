#!/usr/bin/env python3

import argparse
import sys

try:
    import pigpio
except ImportError as exc:
    raise SystemExit(f"pigpio import failed: {exc}")


GPIO = 17


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--mode", required=True, choices=("baseline", "perturb"))
    parser.add_argument("--frames", type=int, default=128)
    parser.add_argument("--perturb-frame", type=int, default=50)
    parser.add_argument("--preamble-count", type=int, default=10)
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


def main() -> int:
    args = parse_args()
    intervals = build_intervals(args)

    pi = pigpio.pi()
    if not pi.connected:
        raise SystemExit("pigpio daemon not reachable")
    pi.set_mode(GPIO, pigpio.OUTPUT)

    half_pulses: list[pigpio.pulse] = []
    for interval in intervals:
        if interval % 2 != 0:
            raise SystemExit(f"interval must be even: {interval}")
        half = interval // 2
        half_pulses.append(pigpio.pulse(1 << GPIO, 0, half))
        half_pulses.append(pigpio.pulse(0, 1 << GPIO, half))

    try:
        pi.write(GPIO, 0)
        pi.wave_clear()
        pi.wave_add_generic(half_pulses)
        wave_id = pi.wave_create()
        if wave_id < 0:
            raise SystemExit(f"wave_create failed: {wave_id}")
        pi.wave_send_once(wave_id)
        while pi.wave_tx_busy():
            pass
        pi.write(GPIO, 0)
        pi.wave_delete(wave_id)
        return 0
    finally:
        pi.wave_clear()
        pi.stop()


if __name__ == "__main__":
    sys.exit(main())
