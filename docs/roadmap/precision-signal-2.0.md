# Precision Signal 2.0 Roadmap

## Project Direction

Precision Signal is a deterministic execution validation system centered on replay, operated through the `precision` CLI against an attached STM32 target over UART.

Precision Signal 2.0 focuses on replay-gap reduction and a cleaner front-facing replay/evidence path.

## Current Foundation

The active retained release foundation for 2.0 is `1.9.1`.

## Replay Model

In Precision Signal, replay means checking retained execution evidence or target behavior against a documented deterministic rule.

The current single-board path uses the STM32F446 UART capture path, retained RPL0 artifacts, and the `precision` CLI comparison workflow.

The current RPL0 witness reduces replay self-reference at the artifact parse/fold layer by checking retained RPL0 data through an independent implementation path.

A two-board path becomes replay-relevant when the second board independently checks actor behavior or retained artifact expectations against the replay rule. If it only records timing or external signals without checking the replay rule, it remains support observation rather than a replay check.

The implemented `dual_stm32_replay_witness_v1` path is replay-relevant because it checks actor-visible behavior against the documented `dual_stm32_pa6_pa1_pair_v0` deterministic rule. A second STM32F446RE observes the actor PA6/PA1 event stream through PB8/PB9, accepts 10007 ordered `TRIGGER, ACK` pairs, and emits a retained PASS witness report. This retained witness supports the replay evidence path, but it does not replace the canonical `precision` CLI replay comparison workflow and is not release authority by default.

## 2.0 Intent

Precision Signal 2.0 reduces replay self-reference in the current STM32F446 UART replay path and makes the front-facing replay/evidence path easier to review.

The work is centered on the `precision` CLI, retained RPL0 artifacts, explicit hardware profiles, and local witness support evidence.

## 2.0 Scope

* `precision` CLI as the canonical replay surface
* STM32F446 target replay path over UART
* explicit hardware profiles and wiring assumptions
* separation of replay, flashing, reset, observation, and retained evidence
* retained artifact contract
* front-facing documentation discipline
* guarded release/tagging hygiene

## Replay Gap to Remedy

The current replay/evidence path has a self-reference gap: the same project defines the retained RPL0 format and provides the main replay/comparison path that checks it.

Precision Signal 2.0 reduces that gap with a small independent RPL0 witness. The witness does not duplicate the replay engine. It independently parses and folds retained RPL0 v1 artifacts so the retained artifact has a second check outside the main CLI comparison path.

This remedy is intentionally narrow:

* keep the `precision` CLI as the canonical operator surface
* keep RPL0 as the retained execution-evidence artifact
* separate artifact identity checks from semantic replay checks
* promote witness output to 2.0 authority only when the retained output is
  checked by `make replay-witness-check VERSION=2.0.0` and that command is
  required by the release validation or guarded tag path

The independent RPL0 witness improves review confidence at the artifact parse/fold layer. It does not replace the CLI comparison workflow. For 2.0, it becomes release authority only as part of the named 2.0 authority chain and only when an enforced release command requires it.

## Core Work Areas

1. CLI-centered replay workflow
2. Hardware profile clarity
3. Reset/flashing semantics
4. Retained evidence contract
5. Release hygiene and verification
6. Front-facing documentation clarity
7. Self-diff reduction / independent replay check

## Hardware Profile Model

Hardware profiles are named contracts that tie a replay or evidence path to a specific hardware and operator context. A profile should define:

* board role
* firmware role
* transport/interface
* wiring assumptions
* generated artifacts
* evidence claim
* claim boundary

Example profile names, not required implemented profiles:

```text
single_stm32_uart_replay_v1
dual_stm32_replay_witness_v1
dual_stm32_external_observer_v1
```

## Replay, Flashing, Reset, Observation, and Evidence Boundaries

The 2.0 documentation must keep these boundaries explicit:

```text
flashing != replay
reset != evidence
transport != target behavior
external observation != actor-internal replay
support instrumentation != release evidence
```

**Flashing** establishes what firmware image is believed to be on the target.

**Reset** establishes the starting or attach condition.

**Replay** checks retained execution evidence or target behavior against a documented deterministic rule.

**Observation** records target or external observer behavior.

**Retained evidence** preserves reviewable artifacts and bounded claims.

## Retained Evidence Model

Retained evidence is a reviewable bundle containing, where applicable:

* command path
* firmware/build identity
* target board/profile
* wiring assumptions
* generated output
* metadata
* manual context files
* claim boundary

Generated artifacts should not be edited after capture. Manual context should be retained separately and clearly identified.

## Replay Evidence Composition

The `precision` CLI remains the canonical replay comparison surface.

Retained RPL0 artifacts remain the retained execution-evidence path. The RPL0 witness reduces self-reference at the artifact parse/fold layer.

The `single_stm32_uart_replay_v1` profile ties replay to the STM32F446 UART target path. The retained `dual_stm32_replay_witness_v1` run adds external hardware rule-checking of actor-visible behavior.

Timing observer evidence remains support observation unless it checks a replay rule.

## Required vs Supporting Evidence

Required evidence for 2.0:

* canonical `precision` CLI replay path documented
* retained `1.9.1` release foundation understood
* retained RPL0 evidence path documented
* independent RPL0 artifact witness documented and promoted only through the
  2.0 authority chain
* replay/flashing/reset/observation/evidence boundaries documented
* hardware profile model documented
* guarded release/tagging process documented

Supporting evidence for 2.0:

* retained dual-STM32 replay witness run
* timing observer evidence
* negative scratch validation

Supporting evidence matters, but it does not block 2.0 unless explicitly promoted into release criteria.

## Proof-Gap Table

| Claim area | Current evidence | What it proves | Required for 2.0? | Remaining gap | Next action |
| ---------- | ---------------- | -------------- | ----------------- | ------------- | ----------- |
| CLI replay comparison | Documented `precision` replay workflow and retained release surface | The canonical operator comparison path is defined and reviewable | Yes | No separate retained `2.0` release bundle yet | Keep the CLI path front-facing and bounded |
| retained `1.9.1` release foundation | Retained `1.9.1` release evidence | 2.0 is building from an understood retained release foundation | Yes | 2.0 has not yet superseded it with a retained bundle | Preserve the foundation boundary in 2.0 docs |
| RPL0 retained artifact path | RPL0 retained artifacts and firmware capture contract | Execution evidence has a retained artifact path | Yes | Artifact evidence is not the whole semantic replay authority | Keep artifact and CLI comparison roles separated |
| independent RPL0 witness | RPL0 witness report for `fw_capture.bin` | The retained artifact parse/fold layer has an independent host-side check | Yes | It is 2.0 authority only when retained output is checked by `make replay-witness-check VERSION=2.0.0` and required by release validation or the guarded tag path | Keep witness promotion tied to the authority chain |
| single-STM32 UART replay profile | `single_stm32_uart_replay_v1` profile | The STM32F446 UART target replay path has a named hardware profile | Yes | Profile clarity does not itself prove target behavior | Keep profile links and boundaries visible |
| dual-STM32 replay witness | Retained run under `artifacts/hil_replay_witness_dual/0001/` | A second STM32F446RE can externally check PA6/PA1 actor behavior against `dual_stm32_pa6_pa1_pair_v0` and accept 10007 ordered `TRIGGER, ACK` pairs | No, supporting retained hardware evidence | It is not release authority by default and does not replace CLI replay comparison | Retain as supporting evidence unless later promoted |
| timing observer evidence | Dual timing observer artifacts | External timing behavior has support observation | No, supporting observation | Timing evidence is not replay authority unless it checks a replay rule | Keep timing claims outside replay authority |
| negative/fault behavior | Negative scratch validation | The witness runner rejects a valid witness `RESULT: FAIL` rather than treating capture success as run success | No, supporting validation | Scratch validation is not retained evidence unless promoted later | Do not treat scratch results as release evidence |
| end-to-end replay/release composition | Roadmap, release surface, replay index, retained evidence docs | The replay/evidence path is finite and reviewable | Yes | Reviewers still need a clear front-facing path across files | Keep this roadmap as the composition map |
| naming/repo positioning | Current project and repo naming | No rename is required before 2.0 | No | Current naming could be revisited only if it forces a misleading claim | Defer naming/repo positioning until after 2.0 unless it blocks truthful 2.0 claims |

## Naming/Repo Positioning

No rename is required before 2.0. The 2.0 work should first settle the replay evidence path and claim boundary. Naming/repo positioning can be revisited after 2.0 unless the current name forces a misleading claim.


## Release Authority Governance

This roadmap is the 2.0 release authority governance surface. It defines the
2.0 release authority chain, the evidence promotion rule, required retained
authority files, and support-evidence exclusions.

For 2.0, authority comes from retained single-board replay evidence plus
host-side independent witness verification. Single-board retained evidence alone
is not sufficient without the host-side witness check.

Dual-board promoted evidence would add independent hardware observation of live
target behavior, but it is not required for the minimal 2.0 release authority
path. Dual-STM32 evidence and two-board observer evidence remain support
evidence or future promotion candidates unless separately promoted under the
authority rule.

The 2.0 release authority chain is:

```text
make gate
make authoritative-replay-cli-tests
make replay-witness-check VERSION=2.0.0
make release-bundle-check VERSION=2.0.0
-> retained docs/verification/releases/2.0.0/{index.md,summary.md,summary.json,fw_capture.bin,rpl0_witness_fw_capture.txt}
-> make release-tag VERSION=2.0.0
```

`make replay-witness-check VERSION=2.0.0` is a planned required 2.0 authority
command. This roadmap documents the requirement but does not implement the
command.

The promoted 2.0 authority evidence is:

```text
docs/verification/releases/2.0.0/fw_capture.bin
docs/verification/releases/2.0.0/rpl0_witness_fw_capture.txt
```

The 2.0 release record also requires:

```text
docs/verification/releases/2.0.0/index.md
docs/verification/releases/2.0.0/summary.md
docs/verification/releases/2.0.0/summary.json
```

Evidence becomes 2.0 release authority only when it:

* directly checks the 2.0 release claim
* has deterministic pass/fail behavior
* is rerunnable or verifiable from retained release inputs
* failure invalidates the release record
* has negative coverage
* is named in the authority chain
* is required by release validation or the guarded tag path

Evidence is authority only when all promotion requirements are satisfied and an
enforced release command requires it. Evidence that is merely retained, linked,
cited, hardware-backed, impressive, or historically important remains support
evidence.

Authority is not inherited from proximity to release work; it is granted only by
explicit inclusion in the authority chain and enforcement by a release command.

The following are outside 2.0 release authority unless separately promoted under
the rule:

```text
bench-check
fw-gate
dual-STM32 evidence
two-board observer evidence
timing capture
BBB orchestration
manual reset evidence
scratch artifacts
latest-run directories
unretained local captures
```

They may remain support, diagnostic, experimental, or historical evidence where
already documented.

## Release Criteria

Precision Signal 2.0 release criteria:

* roadmap and replay boundaries documented
* canonical CLI replay path documented
* hardware profile model documented
* replay/flashing/reset/evidence separation documented
* retained evidence contract documented
* retained verification artifacts mapped to the replay claim
* release/tagging process guarded and documented
* 2.0 authority chain documented with required release commands
* retained `2.0.0` authority files documented
* independent replay check for the STM32F446 UART capture path promoted only
  through the 2.0 authority chain
* support evidence excluded from 2.0 authority unless separately promoted under the governance rule

## Current Criteria Status

The current retained foundation for 2.0 is `1.9.1`. Precision Signal 2.0 does
not yet have a separate retained `2.0.0` release record.

The current documentation and evidence chain satisfies or partially satisfies the 2.0 release criteria as follows:

| Criterion | Current status |
| --- | --- |
| roadmap and replay boundaries documented | satisfied by this roadmap |
| canonical CLI replay path documented | satisfied by README, release surface, replay index, and CLI contract |
| hardware profile model documented | satisfied by the hardware profile model in this roadmap and the active `single_stm32_uart_replay_v1` profile |
| replay/flashing/reset/evidence separation documented | satisfied by roadmap, release surface, firmware capture contract, and hardware profile |
| retained evidence contract documented | satisfied by `docs/verification/releases/index.md` and retained `1.9.1` evidence |
| retained verification artifacts mapped to the replay claim | satisfied for the current `1.9.1` foundation; `2.0.0` requires the retained authority files named in this roadmap |
| release/tagging process guarded and documented | satisfied by retained release mechanics and guarded release tag flow |
| 2.0 authority chain documented with required release commands | satisfied by the Release Authority Governance section in this roadmap |
| retained `2.0.0` authority files documented | satisfied by the Release Authority Governance section in this roadmap; the retained `2.0.0` release record is not created by this roadmap |
| independent replay check promoted through the 2.0 authority chain | satisfied as a documented requirement; `make replay-witness-check VERSION=2.0.0` remains a planned required authority command |
| support evidence excluded from 2.0 authority unless separately promoted under the governance rule | satisfied by the Release Authority Governance section in this roadmap |

The independent RPL0 witness report reduces replay self-reference at the
artifact parse/fold layer. For 2.0, it becomes authority only when its retained
output is checked by `make replay-witness-check VERSION=2.0.0` and that command
is required by release validation or the guarded tag path. The retained
dual-STM32 replay witness is support evidence unless explicitly promoted later.

## Success Definition

Precision Signal 2.0 succeeds if the STM32F446 UART replay path has a clear
canonical CLI comparison path, retained single-board replay evidence, host-side
independent witness verification, and front-facing documentation that lets a
reviewer follow an executable, retained, negative-covered, release/tag-gated
authority path without reconstructing it from scattered files.
