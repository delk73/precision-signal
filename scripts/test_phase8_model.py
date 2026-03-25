#!/usr/bin/env python3
"""Offline contract test for the frozen phase8 signal model.

Simulates the read-then-advance phase accumulator and verifies
the frozen convention: sample = frame_idx & 0xFF.

No hardware required.
"""

STEP = 0x0100_0000
FRAME_COUNT = 10_000


def simulate_read_then_advance(frame_count: int) -> list[int]:
    """Simulate the normalized ISR phase accumulator."""
    phase: int = 0
    samples: list[int] = []
    for _ in range(frame_count):
        sample = (phase >> 24) & 0xFF  # read FIRST
        samples.append(sample)
        phase = (phase + STEP) & 0xFFFF_FFFF  # then advance
    return samples


def main() -> int:
    samples = simulate_read_then_advance(FRAME_COUNT)

    # 1. Frozen formula: sample[i] == i & 0xFF for all frames
    for i, s in enumerate(samples):
        expected = i & 0xFF
        if s != expected:
            print(f"FAIL: sample[{i}] = 0x{s:02X}, expected 0x{expected:02X}")
            return 1

    # 2. Explicit migration gate: frame 0 starts at 0x00
    assert samples[0] == 0x00, f"FAIL: sample[0] = 0x{samples[0]:02X}, expected 0x00"

    # 3. Negative check: old formula (frame_idx + 1) & 0xFF fails at frame 0
    old_formula_frame0 = (0 + 1) & 0xFF  # == 1
    assert samples[0] != old_formula_frame0, (
        f"FAIL: sample[0] matches old convention 0x{old_formula_frame0:02X} — "
        "mixed-contract drift detected"
    )

    # 4. Explicit boundary values
    assert samples[255] == 0xFF, f"FAIL: sample[255] = 0x{samples[255]:02X}, expected 0xFF"
    assert samples[256] == 0x00, f"FAIL: sample[256] = 0x{samples[256]:02X}, expected 0x00"

    print(f"PASS: all {FRAME_COUNT} frames match frozen convention (phase8 := frame_idx & 0xFF)")
    print(f"  sample[0]   = 0x{samples[0]:02X}")
    print(f"  sample[255] = 0x{samples[255]:02X}")
    print(f"  sample[256] = 0x{samples[256]:02X}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
