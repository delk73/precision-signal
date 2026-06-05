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
| independent replay check documented with local witness classification | satisfied as retained support evidence by the RPL0 witness report for `fw_capture.bin` |

The independent RPL0 witness report is retained support evidence. It reduces replay self-reference at the artifact parse/fold layer, but it is not semantic replay authority and does not replace the retained release evidence path.

## Success Definition

Precision Signal 2.0 succeeds if the current STM32F446 UART replay path has an independent retained artifact check, the role of that check is clearly bounded, and the front-facing documentation lets a reviewer follow the replay/evidence path without reconstructing it from scattered files.
