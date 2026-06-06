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
* classify witness output as support evidence, not semantic replay authority

The independent RPL0 witness improves review confidence at the artifact parse/fold layer. It does not replace the CLI comparison workflow or become semantic replay authority unless a later release-evidence change explicitly promotes that classification.

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
* independent RPL0 artifact witness documented
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
| independent RPL0 witness | RPL0 witness report for `fw_capture.bin` | The retained artifact parse/fold layer has an independent support check | Yes | It is support evidence, not semantic replay authority | Keep witness classification bounded |
| single-STM32 UART replay profile | `single_stm32_uart_replay_v1` profile | The STM32F446 UART target replay path has a named hardware profile | Yes | Profile clarity does not itself prove target behavior | Keep profile links and boundaries visible |
| dual-STM32 replay witness | Retained run under `artifacts/hil_replay_witness_dual/0001/` | A second STM32F446RE can externally check PA6/PA1 actor behavior against `dual_stm32_pa6_pa1_pair_v0` and accept 10007 ordered `TRIGGER, ACK` pairs | No, supporting retained hardware evidence | It is not release authority by default and does not replace CLI replay comparison | Retain as supporting evidence unless later promoted |
| timing observer evidence | Dual timing observer artifacts | External timing behavior has support observation | No, supporting observation | Timing evidence is not replay authority unless it checks a replay rule | Keep timing claims outside replay authority |
| negative/fault behavior | Negative scratch validation | The witness runner rejects a valid witness `RESULT: FAIL` rather than treating capture success as run success | No, supporting validation | Scratch validation is not retained evidence unless promoted later | Do not treat scratch results as release evidence |
| end-to-end replay/release composition | Roadmap, release surface, replay index, retained evidence docs | The replay/evidence path is finite and reviewable | Yes | Reviewers still need a clear front-facing path across files | Keep this roadmap as the composition map |
| naming/repo positioning | Current project and repo naming | No rename is required before 2.0 | No | Current naming could be revisited only if it forces a misleading claim | Defer naming/repo positioning until after 2.0 unless it blocks truthful 2.0 claims |

## Naming/Repo Positioning

No rename is required before 2.0. The 2.0 work should first settle the replay evidence path and claim boundary. Naming/repo positioning can be revisited after 2.0 unless the current name forces a misleading claim.

## Release Criteria

Precision Signal 2.0 release criteria:

* roadmap and replay boundaries documented
* canonical CLI replay path documented
* hardware profile model documented
* replay/flashing/reset/evidence separation documented
* retained evidence contract documented
* retained verification artifacts mapped to the replay claim
* release/tagging process guarded and documented
* independent replay check for the STM32F446 UART capture path documented with local witness classification

## Current Criteria Status

The current retained foundation for 2.0 is `1.9.1`. Precision Signal 2.0 does not yet have a separate retained `2.0` release bundle.

The current documentation and evidence chain satisfies or partially satisfies the 2.0 release criteria as follows:

| Criterion | Current status |
| --- | --- |
| roadmap and replay boundaries documented | satisfied by this roadmap |
| canonical CLI replay path documented | satisfied by README, release surface, replay index, and CLI contract |
| hardware profile model documented | satisfied by the hardware profile model in this roadmap and the active `single_stm32_uart_replay_v1` profile |
| replay/flashing/reset/evidence separation documented | satisfied by roadmap, release surface, firmware capture contract, and hardware profile |
| retained evidence contract documented | satisfied by `docs/verification/releases/index.md` and retained `1.9.1` evidence |
| retained verification artifacts mapped to the replay claim | satisfied for the current `1.9.1` foundation |
| release/tagging process guarded and documented | satisfied by retained release mechanics and guarded release tag flow |
| independent replay check documented with local witness classification | satisfied at the artifact layer by the RPL0 witness report for `fw_capture.bin`; strengthened at the hardware support layer by the retained dual-STM32 replay witness run under `artifacts/hil_replay_witness_dual/0001/` |

The independent RPL0 witness report is retained support evidence. It reduces replay self-reference at the artifact parse/fold layer, but it is not semantic replay authority and does not replace the retained release evidence path. The retained dual-STM32 replay witness is also support evidence unless explicitly promoted later.

## Success Definition

Precision Signal 2.0 succeeds if the STM32F446 UART replay path has a clear canonical CLI comparison path, independent retained artifact-layer checking, bounded supporting hardware witness evidence, and front-facing documentation that lets a reviewer follow the replay/evidence path without reconstructing it from scattered files.
