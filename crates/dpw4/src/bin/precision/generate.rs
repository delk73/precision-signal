use super::{GenerateArgs, ShapeArg};
use crate::common;
use crate::common::CliError;
use dpw4::{
    math, signal_pipe, DpwGain, OscState, Pulse, Sawtooth, Scalar, SignalFrameHeader, Square,
    TriangleDPW1, TriangleDPW4, BIT_DEPTH_32,
};
use std::io::{self, Write};

pub(crate) fn run_generate(args: GenerateArgs) -> Result<(), CliError> {
    if args.seconds == Some(0) {
        return Err(CliError::User("--seconds must be greater than 0".to_string()));
    }

    if args.container_wav && args.seconds.is_none() {
        return Err(CliError::User(
            "--container-wav requires --seconds to be set.".to_string(),
        ));
    }

    let mut osc = OscState::new();
    let freq_scalar = Scalar::from_num(args.freq);
    let rate_scalar = Scalar::from_num(args.rate);
    let two_pi_scalar = math::TWO_PI;
    let phase_inc = (freq_scalar / rate_scalar) * two_pi_scalar;

    let linear = libm::pow(10.0, args.gain / 20.0);
    let mut gain_f64 = linear * (1u64 << 63) as f64;
    let mut gain_exp = 0;

    while gain_f64 >= 18446744073709551616.0 {
        gain_f64 *= 0.5;
        gain_exp += 1;
    }

    let gain = DpwGain::new(gain_f64 as u64, gain_exp, 0, 0);

    let mut handle = common::open_output(&args.out)?;

    if args.container_wav {
        let seconds = args.seconds.ok_or_else(|| {
            CliError::User("--container-wav requires --seconds".to_string())
        })?;
        let total_samples = args.rate as u64 * seconds;
        let data_bytes = total_samples * 4;
        write_wav_header(&mut handle, args.rate, data_bytes as u32).map_err(CliError::Io)?;
    } else {
        let header = SignalFrameHeader::new(0, args.rate);
        handle.write_all(&header.to_bytes()).map_err(CliError::Io)?;
    }

    let mut buffer = [0i32; 512];
    let mut phases = [Scalar::ZERO; 512];
    let mut current_phase = Scalar::ZERO;
    let mut samples_remaining = args
        .seconds
        .map(|s| s * args.rate as u64)
        .unwrap_or(u64::MAX);

    while samples_remaining > 0 {
        let chunk_size = std::cmp::min(buffer.len() as u64, samples_remaining) as usize;

        for i in 0..chunk_size {
            phases[i] = current_phase;
            current_phase += phase_inc;
            if current_phase >= two_pi_scalar {
                current_phase -= two_pi_scalar;
            }
        }

        match args.shape {
            ShapeArg::Saw => signal_pipe::<Sawtooth>(
                &mut osc,
                &phases[0..chunk_size],
                &gain,
                &mut buffer[0..chunk_size],
            ),
            ShapeArg::Square => signal_pipe::<Square>(
                &mut osc,
                &phases[0..chunk_size],
                &gain,
                &mut buffer[0..chunk_size],
            ),
            ShapeArg::Triangle => {
                signal_pipe::<TriangleDPW4>(
                    &mut osc,
                    &phases[0..chunk_size],
                    &gain,
                    &mut buffer[0..chunk_size],
                );
            }
            ShapeArg::TriangleDpw1 => {
                if samples_remaining
                    == args
                        .seconds
                        .map(|s| s * args.rate as u64)
                        .unwrap_or(u64::MAX)
                {
                    eprintln!("⚠️  ADVISORY: Triangle (DPW1 Naive) is non-band-limited and will alias at high frequencies.");
                }
                signal_pipe::<TriangleDPW1>(
                    &mut osc,
                    &phases[0..chunk_size],
                    &gain,
                    &mut buffer[0..chunk_size],
                );
            }
            ShapeArg::Pulse => {
                osc.duty = Scalar::from_num(0.1);
                signal_pipe::<Pulse>(
                    &mut osc,
                    &phases[0..chunk_size],
                    &gain,
                    &mut buffer[0..chunk_size],
                );
            }
        }

        let mut byte_buffer = [0u8; 512 * 4];
        for (i, &sample) in buffer[0..chunk_size].iter().enumerate() {
            byte_buffer[i * 4..(i + 1) * 4].copy_from_slice(&sample.to_le_bytes());
        }

        if let Err(e) = handle.write_all(&byte_buffer[0..chunk_size * 4]) {
            if e.kind() == io::ErrorKind::BrokenPipe {
                break;
            }
            return Err(CliError::Io(e));
        }

        if args.seconds.is_some() {
            samples_remaining -= chunk_size as u64;
        }
    }
    handle.flush().map_err(CliError::Io)?;
    Ok(())
}

fn write_wav_header<W: Write>(writer: &mut W, sample_rate: u32, data_bytes: u32) -> io::Result<()> {
    let total_file_size = 36 + data_bytes;
    writer.write_all(b"RIFF")?;
    writer.write_all(&total_file_size.to_le_bytes())?;
    writer.write_all(b"WAVE")?;
    writer.write_all(b"fmt ")?;
    writer.write_all(&16u32.to_le_bytes())?;
    writer.write_all(&1u16.to_le_bytes())?;
    writer.write_all(&1u16.to_le_bytes())?;
    writer.write_all(&sample_rate.to_le_bytes())?;
    let byte_rate = sample_rate * 4;
    writer.write_all(&byte_rate.to_le_bytes())?;
    writer.write_all(&4u16.to_le_bytes())?;
    writer.write_all(&(BIT_DEPTH_32 as u16).to_le_bytes())?;
    writer.write_all(b"data")?;
    writer.write_all(&data_bytes.to_le_bytes())?;
    Ok(())
}
