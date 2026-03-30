# NON-NORMATIVE / EXPERIMENTAL

## 2026-03-30 — flash attach regression [WIP-013]
Status: active
Owner: signal

Problem
On the same PC, with the same NUCLEO-F446RE board, the same `st-flash 1.7.0`, and the same `make flash-ur` path, attach behavior is now intermittent. A path that was previously reliable for hot-plug and repeated reflashing can no longer be assumed reliable, because back-to-back runs can succeed and then fail without an intentional tooling or command change in this pass.

Hypothesis
Target-side SWD/reset attach window appears unstable after power-on. Exact cause unknown. One possible mitigation, if later validated, is widening attach margin in firmware via an early startup delay.

Constraints
- non-normative only
- no repo-wide flashing policy change in this pass
- no Makefile patch in this pass
- no firmware patch in this pass
- preserve operator evidence exactly

Planned Artifacts
- `docs/wip/flash_attach_regression.md`
- observed shell transcript source from the local terminal session, if retained separately

Evidence Produced
- `make flash-ur`
- `st-flash --connect-under-reset --freq=200K --reset write ...`
- `Soft reset failed: timeout`
- `Can not connect to target`
- `st-info --probe`
- `chipid: 0x0000`
- `flash: 0`
- `descr: unknown device`
- Back-to-back `make flash-ur` runs:
- `success`
- `success`
- `failure`
- Manual reset-timed flashing succeeded when automatic attach failed.

Next Decision
Does a fixed early startup delay in firmware restore reliable attach across cold power cycles?

Promotion Path
If validated, summarize in `docs/wip/experiment_log.md` and convert the mitigation itself into a separate modification pass.
