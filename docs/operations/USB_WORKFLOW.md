# USB Workflow (xtask)

`xtask` provides a USB-only track:

- `cargo xtask usb doctor --board <id-or-path>`
- `cargo xtask usb flash --board <id-or-path> --elf <path/to/fw.elf> [--log <path>] [--execute]`
- `cargo xtask usb debug --board <id-or-path> [--execute]`

## Board argument semantics

`--board <id-or-path>` resolves as:

- if the value contains `/` or `\`, or ends with `.toml`: treat as a path
- path-style values are canonicalized before containment checks (canonicalization resolves symlinks)
- symlink rule: if the board path is, or traverses, a symlink that resolves outside repo root, resolution fails with `FAIL board.path_escape`
- all path-style values (relative or absolute) must resolve to a descriptor under repo root; escapes fail with `FAIL board.path_escape`
- convenience: use copied or vendored board descriptors inside the repo instead of symlinks to external locations
- otherwise: treat as board id and resolve to `boards/<id>.toml` relative to repo root

Missing board files are reported as `FAIL` checks (exit code 1), not usage errors.

## Descriptor format (real TOML)

Board descriptors are strict TOML with this schema:

- required: `chip`, `flash_base`, `preferred_backend`
- `preferred_backend`: `stlink` or `openocd`
- optional: `vid`, `pid`, `openocd_interface_cfg`, `openocd_target_cfg`
- optional table: `[ports]`
- `[ports]` keys: `gdb`, `stutil`, `openocd_tcl`, `openocd_telnet`

Unknown keys or unknown tables fail parsing.

## Port semantics

- `stlink` uses only `ports.stutil`
- `openocd` uses `ports.gdb`, `ports.openocd_tcl`, `ports.openocd_telnet`
- `ports.gdb` is OpenOCD-only and is not checked for `stlink`

## Output contract

- default text output: `PASS|WARN|FAIL <check_id>: <message>` with optional `(hint: ...)`
- `--json`: JSON-only stdout, schema version `usb-report.v1`
- `--verbose`: deterministic diagnostic lines prefixed with `TRACE <event_id>: <message>`
- checks are sorted deterministically by `check_id` (then `status`/`message`/`hint`); do not assume chronological order

JSON schema is defined at `docs/usb/usb-report.v1.schema.json`:

- checks always include `hint` as `string|null`
- events are `{ "kind": "...", "text": "..." }`

`usb debug` is interactive when `--execute` is used. Deterministic capture of runtime process output is intentionally not provided; `--verbose` emits deterministic `TRACE spawn_command_line: ...` lines only.

## Exit codes

- report contains any `FAIL` check: exit `1`
- report contains only `PASS`/`WARN`: exit `0`
- usage/argument shape errors: exit `2`

Missing `--board` and missing subcommand are usage errors. Missing board or missing flash input/file are `FAIL` report cases.

## Flash semantics

- `usb flash` accepts `--elf <path>` (required in normal flow)
- compatibility mode: `--image <path>` is deprecated and accepted only for `stlink` with `.bin` input; emits `WARN cli.deprecated_image_flag`
- providing both `--elf` and `--image` is an argument-shape error (usage exit `2`)
- providing neither `--elf` nor `--image` emits `FAIL cli.elf_required` (exit `1`)
- if `--execute` is absent: report + deterministic command plan only; no process spawn

Backend behavior:

- `stlink` with `--elf`: runs `rust-objcopy -O binary <elf> target/xtask/usb/<board>.bin`, then `st-flash write <bin> <flash_base>`, then best-effort cleanup of generated bin
- `openocd`: programs ELF directly in a single OpenOCD invocation (no BIN generation)
- `--execute` reports one deterministic per-step check id:
  - `usb.flash.execute_step.<index>.<command>` (for example `usb.flash.execute_step.00.rust-objcopy`)

Runlog behavior for `usb flash --execute`:

- default path: `docs/audits/runlogs/<YYYYMMDD>_usb_flash.txt`
- override: `--log <path>`
- deterministic content order: command line(s), captured stdout/stderr, cleanup notes

## Audit transcript integrity

When recording command transcripts for pre-release audits, enable strict shell behavior so pipeline commands do not mask tool failures:

```bash
set -euo pipefail
```

If you capture JSON/text via `tee`, record the producer command exit status explicitly:

```bash
cargo xtask usb doctor --board missing --json | tee /tmp/usb.json
echo "cargo_exit=${PIPESTATUS[0]}"
```

## Determinism and side effects

- `usb doctor` is read-only
- wall-clock date is used only to form the default flash runlog path (`<YYYYMMDD>`), not for check logic or ordering
