# Release Bundle Summary

Version: 2.0.0
Generated: 2026-06-09T18:10:35Z
Capture source: artifacts/run.bin

## Authority Validation Commands

```bash
make gate
make authoritative-replay-cli-tests
make replay-witness-check VERSION=2.0.0
make release-bundle-check VERSION=2.0.0
```

## Quick Validate Output

```text
   Compiling precision-cli v2.0.0 (./crates/precision-cli)
    Finished `release` profile [optimized] target(s) in 4.03s
     Running `target/release/sig-util validate --mode quick`
PASS version_consistency: workspace=2.0.0 lock=2.0.0
PASS toolchain_pin: channel=1.91.1
PASS header_stream_integrity: header-only stream valid (100 frames)
WARN non_normative_canary: id=phase_wrap_440 actual=957018c2fdf8bc8c20ddb61d9d7412a8145083cad930356da889a7580e060db6
PASS determinism_bit_exact:
    phase_wrap_440.canon.sig
      Golden:   Unpinned
      Actual:   957018c2fdf8bc8c20ddb61d9d7412a8145083cad930356da889a7580e060db6
    pulse_relational_8k.canon.sig
      Golden:   953d0533d83ba3573e7f7948199f5cd66adc6f5c43fdedf9ceb2ffe45fdb324c
      Actual:   953d0533d83ba3573e7f7948199f5cd66adc6f5c43fdedf9ceb2ffe45fdb324c
    saw_20_headroom.canon.sig
      Golden:   143d4613ab993dbacc5e5f735d4985ee1a188b8b18d1a906dd44ae621086993e
      Actual:   143d4613ab993dbacc5e5f735d4985ee1a188b8b18d1a906dd44ae621086993e
    triangle_linearity_1k.canon.sig
      Golden:   74ef101edcdcbffb20be729ed503eaa04dd47301ec2f3b26f76786879850dbfd
      Actual:   74ef101edcdcbffb20be729ed503eaa04dd47301ec2f3b26f76786879850dbfd
    sine_linearity_1k.canon.sig
      Golden:   00a9577a82ffe1c9b6a05ee9bcd6f93947463bd901ef05d528e2a7704006bfbc
      Actual:   00a9577a82ffe1c9b6a05ee9bcd6f93947463bd901ef05d528e2a7704006bfbc
    long_run_0_1hz.canon.sig
      Golden:   1e116cc76d4c460f5eab421a30f20b42157fe516d53440baca3c7dabdb92d420
      Actual:   1e116cc76d4c460f5eab421a30f20b42157fe516d53440baca3c7dabdb92d420
    master_sweep_20_20k.canon.sig
      Golden:   0ac058b60498cbfb129d4a35b37f2f6c785752fa888c6c4367b0608a4ef825ea
      Actual:   0ac058b60498cbfb129d4a35b37f2f6c785752fa888c6c4367b0608a4ef825ea

VERIFICATION PASSED
```

## Hashes

- fw_capture.bin: sha256:f79e71d6ed645f6bc9f7c3d2b4a8980e0a8cee11cc17082e649966ffba20e765
- rpl0_witness_fw_capture.txt: sha256:0ef79977e4f6c19f89b1b46ddc7fd91fa1e51c63fd30cbf083adc52c785f961e
- summary.md: sha256:237e9ca26dd4e2459b06edb1fee1edf5f302c8d6db1363dbc810ca2aaf1ea30c
- summary.json: sha256:fc4dd4968a307b4de379ecaa12414934cdcc312cacb3574997d96d5e1898e076
- index.md: sha256:986047174537e60a60d307b27e2b2de8302dadae7e208cd280d41def72ac4af9
