use criterion::{
    black_box, criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion, Throughput,
};
use dpw4::{tick_dpw4, tick_shape, Dpw4State, DpwGain, OscState};
use geom_signal::{math, Scalar};
use std::time::Duration;

// NOTE: Phase representation differs by kernel:
// - DPW4 saw uses u32 accumulator (hardware-aligned path)
// - Pulse/Sine use Scalar radians (shape pipeline)

// ARCHITECTURAL INVARIANTS: Named Shape Constants
const SHAPE_PULSE: u32 = 1;
const SHAPE_SINE: u32 = 4;

const UNIT_GAIN: u64 = 1u64 << 63;

const _: fn() = || {
    fn assert_copy<T: Copy>() {}
    assert_copy::<DpwGain>();
};

struct LatchedDpw4 {
    gain: DpwGain,
    phase_inc: u32,
}

struct LatchedPulse {
    gain: DpwGain,
    phase_inc: Scalar,
    shape: u32,
    duty: Scalar,
}

struct LatchedSine {
    gain: DpwGain,
    phase_inc: Scalar,
    shape: u32,
}

const BATCH_SIZE: usize = 256;

/// Configuration helper to deduplicate Criterion setup
fn configure_group<M: criterion::measurement::Measurement>(
    group: &mut criterion::BenchmarkGroup<'_, M>,
    elements: u64,
) {
    group.throughput(Throughput::Elements(elements));
    group.warm_up_time(Duration::from_secs(1));
    group.measurement_time(Duration::from_secs(3));
    group.sample_size(50);
}

/// Inner-loop throughput benchmark for DPW4 Sawtooth.
/// Represents the canonical audio-rate sample-step workload.
/// NOTE: DPW4 sawtooth benchmarks use a u32 phase accumulator.
fn bench_dpw4_sawtooth_sample_step(c: &mut Criterion) {
    let mut group = c.benchmark_group("dpw4_sawtooth");
    configure_group(&mut group, BATCH_SIZE as u64);

    let gain_direct = DpwGain::new(UNIT_GAIN, 0, 0, 0);
    let freq = 440u64;
    let rate = 48000u64;
    let phase_inc_val = ((freq << 32) / rate) as u32;

    let gain_latched = gain_direct; // Copy: explicit intent for latched params
    let params = LatchedDpw4 {
        gain: gain_latched,
        phase_inc: phase_inc_val,
    };

    group.bench_function("direct", |b| {
        // Direct: constants captured from environment, no black_box on inputs.
        // Setup: fresh state/phase per batch.
        b.iter_batched(
            || (Dpw4State::new(), 0u32),
            |(mut state, mut phase_u32)| {
                // Hoist captures to locals to ensure register allocation (helper for LICM)
                let phase_inc_const = phase_inc_val;
                let gain_const = &gain_direct;

                let mut acc: i64 = 0;
                for _ in 0..BATCH_SIZE {
                    let sample = tick_dpw4(&mut state, phase_u32, gain_const);
                    phase_u32 = phase_u32.wrapping_add(phase_inc_const);
                    acc = acc.wrapping_add(sample as i64);
                }
                black_box(acc)
            },
            BatchSize::SmallInput,
        )
    });

    group.bench_function("latched", |b| {
        // Latched: params black_box'd once per batch (hoisted).
        // No black_box inside the inner loop.
        b.iter_batched(
            || (Dpw4State::new(), 0u32),
            |(mut state, mut phase_u32)| {
                let inc = black_box(params.phase_inc);
                let g = black_box(&params.gain);
                let mut acc: i64 = 0;

                for _ in 0..BATCH_SIZE {
                    let sample = tick_dpw4(&mut state, phase_u32, g);
                    phase_u32 = phase_u32.wrapping_add(inc);
                    acc = acc.wrapping_add(sample as i64);
                }
                black_box(acc)
            },
            BatchSize::SmallInput,
        )
    });

    group.finish();
}

/// Inner-loop throughput benchmark for DPW4 Pulse.
/// Represents the differential two-saw workload.
/// NOTE: Pulse and sine benchmarks use Scalar phase in radians.
fn bench_dpw4_pulse_sample_step(c: &mut Criterion) {
    let mut group = c.benchmark_group("tick_shape_pulse");
    configure_group(&mut group, BATCH_SIZE as u64);

    let gain_direct = DpwGain::new(UNIT_GAIN, 0, 0, 0);
    let phase_inc_val = (Scalar::from_num(440) / Scalar::from_num(48000)) * math::TWO_PI;
    let duty_val = Scalar::from_num(0.5);

    let gain_latched = gain_direct; // Copy: explicit intent for latched params
    let params = LatchedPulse {
        gain: gain_latched,
        phase_inc: phase_inc_val,
        shape: SHAPE_PULSE,
        duty: duty_val,
    };

    group.bench_function("direct", |b| {
        b.iter_batched(
            || (OscState::new(), Scalar::ZERO),
            |(mut state, mut phase)| {
                // Hoist captures
                let gain_const = &gain_direct;
                let phase_inc_const = phase_inc_val;
                let duty_const = duty_val;

                state.duty = duty_const;
                let mut acc: i64 = 0;
                for _ in 0..BATCH_SIZE {
                    let sample = tick_shape(&mut state, phase, SHAPE_PULSE, gain_const);
                    phase += phase_inc_const;
                    if phase >= math::TWO_PI {
                        phase -= math::TWO_PI;
                    }
                    acc = acc.wrapping_add(sample as i64);
                }
                black_box(acc)
            },
            BatchSize::SmallInput,
        )
    });

    group.bench_function("latched", |b| {
        b.iter_batched(
            || (OscState::new(), Scalar::ZERO),
            |(mut state, mut phase)| {
                let inc = black_box(params.phase_inc);
                let g = black_box(&params.gain);
                let shape = black_box(params.shape);
                let d = black_box(params.duty);

                // Hoist duty state update outside hot loop
                state.duty = d;
                let mut acc: i64 = 0;

                for _ in 0..BATCH_SIZE {
                    let sample = tick_shape(&mut state, phase, shape, g);
                    phase += inc;
                    if phase >= math::TWO_PI {
                        phase -= math::TWO_PI;
                    }
                    acc = acc.wrapping_add(sample as i64);
                }
                black_box(acc)
            },
            BatchSize::SmallInput,
        )
    });

    group.finish();
}

/// Inner-loop throughput benchmark for Sine (CORDIC).
/// Represents the sin_cos_fast + scaling workload.
fn bench_sine_sample_step(c: &mut Criterion) {
    let mut group = c.benchmark_group("tick_shape_sine");
    configure_group(&mut group, BATCH_SIZE as u64);

    let gain_direct = DpwGain::new(UNIT_GAIN, 0, 0, 0);
    let phase_inc_val = (Scalar::from_num(440) / Scalar::from_num(48000)) * math::TWO_PI;

    let gain_latched = gain_direct; // Copy: explicit intent for latched params
    let params = LatchedSine {
        gain: gain_latched,
        phase_inc: phase_inc_val,
        shape: SHAPE_SINE,
    };

    group.bench_function("direct", |b| {
        b.iter_batched(
            || (OscState::new(), Scalar::ZERO),
            |(mut state, mut phase)| {
                // Hoist captures
                let gain_const = &gain_direct;
                let phase_inc_const = phase_inc_val;

                let mut acc: i64 = 0;
                for _ in 0..BATCH_SIZE {
                    let sample = tick_shape(&mut state, phase, SHAPE_SINE, gain_const);
                    phase += phase_inc_const;
                    if phase >= math::TWO_PI {
                        phase -= math::TWO_PI;
                    }
                    acc = acc.wrapping_add(sample as i64);
                }
                black_box(acc)
            },
            BatchSize::SmallInput,
        )
    });

    group.bench_function("latched", |b| {
        b.iter_batched(
            || (OscState::new(), Scalar::ZERO),
            |(mut state, mut phase)| {
                let inc = black_box(params.phase_inc);
                let g = black_box(&params.gain);
                let shape = black_box(params.shape);
                let mut acc: i64 = 0;

                for _ in 0..BATCH_SIZE {
                    let sample = tick_shape(&mut state, phase, shape, g);
                    phase += inc;
                    if phase >= math::TWO_PI {
                        phase -= math::TWO_PI;
                    }
                    acc = acc.wrapping_add(sample as i64);
                }
                black_box(acc)
            },
            BatchSize::SmallInput,
        )
    });

    group.finish();
}

/// Block-level throughput benchmark for DPW4 Sawtooth.
/// Measures nanoseconds per block for a fixed 128-sample block (common audio buffer size).
fn bench_dpw4_sawtooth_block(c: &mut Criterion) {
    let mut group = c.benchmark_group("dpw4_block");
    const BLOCK_SIZE: usize = 128;
    configure_group(&mut group, BLOCK_SIZE as u64);

    let gain_direct = DpwGain::new(UNIT_GAIN, 0, 0, 0);
    let freq = 440u64;
    let rate = 48000u64;
    let phase_inc_val = ((freq << 32) / rate) as u32;
    let gain_latched = gain_direct; // Copy: explicit intent for latched params
    let params = LatchedDpw4 {
        gain: gain_latched,
        phase_inc: phase_inc_val,
    };

    group.bench_with_input(
        BenchmarkId::new("direct", BLOCK_SIZE),
        &BLOCK_SIZE,
        |b, &_size| {
            b.iter_batched(
                || (Dpw4State::new(), 0u32, [0i32; BLOCK_SIZE]),
                |(mut state, mut phase_u32, mut output)| {
                    // Hoist captures
                    let phase_inc_const = phase_inc_val;
                    let gain_const = &gain_direct;

                    // Direct: constants captured, no input black_box
                    // Sink: output write + acc, black_box both at end
                    let mut acc: i64 = 0;
                    for out in output.iter_mut().take(BLOCK_SIZE) {
                        *out = tick_dpw4(&mut state, phase_u32, gain_const);
                        phase_u32 = phase_u32.wrapping_add(phase_inc_const);
                        acc = acc.wrapping_add(*out as i64);
                    }
                    black_box(acc);
                    black_box(output)
                },
                BatchSize::SmallInput,
            )
        },
    );

    group.bench_with_input(
        BenchmarkId::new("latched", BLOCK_SIZE),
        &BLOCK_SIZE,
        |b, &_size| {
            b.iter_batched(
                || (Dpw4State::new(), 0u32, [0i32; BLOCK_SIZE]),
                |(mut state, mut phase_u32, mut output)| {
                    // Latched: inc/g hoisted once per block closure
                    // Sink: matches direct (output + acc, black_box both)
                    let inc = black_box(params.phase_inc);
                    let g = black_box(&params.gain);
                    let mut acc: i64 = 0;

                    for out in output.iter_mut().take(BLOCK_SIZE) {
                        *out = tick_dpw4(&mut state, phase_u32, g);
                        phase_u32 = phase_u32.wrapping_add(inc);
                        acc = acc.wrapping_add(*out as i64);
                    }
                    black_box(acc);
                    black_box(output)
                },
                BatchSize::SmallInput,
            )
        },
    );

    group.finish();
}

criterion_group!(
    benches,
    bench_dpw4_sawtooth_sample_step,
    bench_dpw4_pulse_sample_step,
    bench_sine_sample_step,
    bench_dpw4_sawtooth_block
);
criterion_main!(benches);
