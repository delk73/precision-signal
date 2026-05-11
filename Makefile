# Deterministic shell path for this repo. If your host uses a different bash
# location (for example /bin/bash), invoke make with SHELL=/path/to/bash.
SHELL := /usr/bin/bash
.ONESHELL:
.SHELLFLAGS := --noprofile --norc -euo pipefail -c

PYTHON := python3
CARGO := cargo
MAKE_NO_PRINT := $(MAKE) --no-print-directory
MAKE_CMD := $(MAKE)
RELEASE_ROOT := docs/verification/releases
RELEASE_DIR := $(RELEASE_ROOT)/$(VERSION)
DPW4_PKG := dpw4
CLI_FEATURE := cli
SIG_UTIL_BIN := sig-util
FW_PKG := replay-fw-f446
FW_TARGET := thumbv7em-none-eabihf
FW_ELF := target/$(FW_TARGET)/debug/$(FW_PKG)
FW_BIN := target/$(FW_TARGET)/debug/$(FW_PKG).bin
FLASH_ADDR := 0x08000000
STFLASH ?= st-flash
FLASH_HEAD := target/flash-head.bin
FLASH_FULL := target/flash-full.bin
SERIAL ?= /dev/ttyACM0
SIGNAL_BASELINE_CSV ?= baseline.csv
SIGNAL_OBSERVED_CSV ?= observed.csv
SIGNAL_FRAMES ?= 128
SIGNAL_PERTURB_FRAME ?= 50
SIGNAL_REPEAT_SECONDS ?=
FW_GATE_RESET_MODE ?= manual
FW_CAPTURE_TIMEOUT ?= 10
REPLAY_CAPTURE_TIMEOUT ?= 60
FW_CAPTURE_DIR ?= artifacts/fw_capture_runs
FW_REPEAT_DIR ?= artifacts/fw_repeat_runs
REPLAY_SIGNAL_MODEL ?= phase8
REPLAY_BASELINE ?= artifacts/baseline.bin
REPLAY_RUN ?= artifacts/run.bin
REPLAY_REPEAT_RUNS ?= 5
REPLAY_REPEAT_DIR ?= artifacts/replay_runs
BENCH_CHECK_FW_ARTIFACTS ?= required
RELEASE_PROOF_FIRMWARE ?= 1
AUDIT_TARGET ?= substrate://probe/default
AUDIT_BIN := ./target/debug/substrate_probe
AUDIT_FIXED_RUN_ID := AUDIT_COLLISION
ARTIFACT_TOOL := $(PYTHON) scripts/artifact_tool.py
XTASK_WORKFLOW := $(CARGO) run --quiet -p xtask -- workflow
DEMO_V2_DIR := artifacts/demo_persistent
DEMO_V2_FIXTURE_A := $(DEMO_V2_DIR)/run_A.rpl
DEMO_V2_FIXTURE_B := $(DEMO_V2_DIR)/run_B.rpl
DEMO_V2_CAPTURED_A := $(DEMO_V2_DIR)/run_A_captured.rpl
DEMO_V2_CAPTURED_B := $(DEMO_V2_DIR)/run_B_captured.rpl
DEMO_V2_CAPTURE_A_DIR := $(DEMO_V2_DIR)/a_capture
DEMO_V2_CAPTURE_B_DIR := $(DEMO_V2_DIR)/b_capture
DEMO_V2_CAPTURE_A_TMP := $(DEMO_V2_CAPTURE_A_DIR)/run_01.bin
DEMO_V2_CAPTURE_B_TMP := $(DEMO_V2_CAPTURE_B_DIR)/run_01.bin
DEMO_V2_RUNLOG_DIR := docs/audits/runlogs
DEMO_V3_DIR := artifacts/demo_v3
DEMO_V3_TRANSIENT_A := $(DEMO_V3_DIR)/transient_A.rpl
DEMO_V3_TRANSIENT_B := $(DEMO_V3_DIR)/transient_B.rpl
DEMO_V3_OFFSET_A := $(DEMO_V3_DIR)/offset_A.rpl
DEMO_V3_OFFSET_B := $(DEMO_V3_DIR)/offset_B.rpl
DEMO_V3_RATE_A := $(DEMO_V3_DIR)/rate_A.rpl
DEMO_V3_RATE_B := $(DEMO_V3_DIR)/rate_B.rpl
DEMO_V3_RUNLOG_DIR := docs/audits/runlogs
DEMO_V4_DIR := artifacts/demo_v4
DEMO_V4_HEADER_A := $(DEMO_V4_DIR)/header_schema_A.rpl
DEMO_V4_HEADER_B := $(DEMO_V4_DIR)/header_schema_B.rpl
DEMO_V4_HEADER_SAMPLE_A := $(DEMO_V4_DIR)/header_schema_sample_payload_A.rpl
DEMO_V4_HEADER_SAMPLE_B := $(DEMO_V4_DIR)/header_schema_sample_payload_B.rpl
DEMO_V4_TIMER_A := $(DEMO_V4_DIR)/timer_delta_A.rpl
DEMO_V4_TIMER_B := $(DEMO_V4_DIR)/timer_delta_B.rpl
DEMO_V4_IRQ_A := $(DEMO_V4_DIR)/irq_state_A.rpl
DEMO_V4_IRQ_B := $(DEMO_V4_DIR)/irq_state_B.rpl
DEMO_V4_SAMPLE_A := $(DEMO_V4_DIR)/sample_payload_A.rpl
DEMO_V4_SAMPLE_B := $(DEMO_V4_DIR)/sample_payload_B.rpl
DEMO_V4_MIXED_A := $(DEMO_V4_DIR)/mixed_A.rpl
DEMO_V4_MIXED_B := $(DEMO_V4_DIR)/mixed_B.rpl
DEMO_V4_RUNLOG_DIR := docs/audits/runlogs
DEMO_V5_DIR := artifacts/demo_v5
DEMO_V5_HEALING_A := $(DEMO_V5_DIR)/self_healing_A.rpl
DEMO_V5_HEALING_B := $(DEMO_V5_DIR)/self_healing_B.rpl
DEMO_V5_BOUNDED_A := $(DEMO_V5_DIR)/bounded_persistent_A.rpl
DEMO_V5_BOUNDED_B := $(DEMO_V5_DIR)/bounded_persistent_B.rpl
DEMO_V5_GROWTH_A := $(DEMO_V5_DIR)/monotonic_growth_A.rpl
DEMO_V5_GROWTH_B := $(DEMO_V5_DIR)/monotonic_growth_B.rpl
DEMO_V5_TRANSITION_A := $(DEMO_V5_DIR)/region_transition_A.rpl
DEMO_V5_TRANSITION_B := $(DEMO_V5_DIR)/region_transition_B.rpl
DEMO_V5_RUNLOG_DIR := docs/audits/runlogs
DEMO_CAPTURED_DIR := artifacts/demo_captured
DEMO_CAPTURED_A := $(DEMO_CAPTURED_DIR)/run_A.rpl
DEMO_CAPTURED_B := $(DEMO_CAPTURED_DIR)/run_B.rpl
DEMO_CAPTURED_A_DIR := $(DEMO_CAPTURED_DIR)/a_capture
DEMO_CAPTURED_B_DIR := $(DEMO_CAPTURED_DIR)/b_capture
DEMO_CAPTURED_A_TMP := $(DEMO_CAPTURED_A_DIR)/run_01.bin
DEMO_CAPTURED_B_TMP := $(DEMO_CAPTURED_B_DIR)/run_01.bin

# Firmware feature flags (space-separated, passed to cargo --features).
# Example:
#   make FW_FEATURES=debug-irq-count flash-ur
FW_FEATURES ?=

# Internal helper: expands to "--features a,b,c" or empty.
FW_FEATURES_ARG := $(if $(strip $(FW_FEATURES)),--features $(subst $(space),$(comma),$(strip $(FW_FEATURES))),)
space :=
space +=
comma := ,

.PHONY: help help-all help-demos help-firmware fixture-drift-check shell-check stflash-check bench-check fw fw-bin flash flash-verify flash-compare flash-ur flash-verify-ur flash-compare-ur demo-signal demo-signal-flash demo-signal-host-baseline demo-signal-host-perturb demo-signal-pi-baseline demo-signal-pi-perturb demo-signal-diff fw-capture-check fw-repeat-check rpl0-replay-check rpl0-replay-repeat-check rpl0-replay-repeat-auto fw-gate firmware-release-summary firmware-release-check fw-release-archive-current fw-release-archive release release-proof release-summary release-1.7.0 release-1.8.0 release-bundle release-bundle-check capture-demo-A capture-demo-B demo-captured-verify demo-captured-release demo-divergence demo-v2-capture demo-v2-fixture-verify demo-v2-verify demo-v2-audit-pack demo-v2-record demo-v3-verify demo-v3-audit-pack demo-v3-record demo-v3-release demo-v4-verify demo-v4-audit-pack demo-v4-record demo-v4-release demo-v5-verify demo-v5-audit-pack demo-v5-record demo-v5-release demo-evidence-package replay-demo-audit debug-session tim2-smoke doc-link-check check-workspace test authoritative-replay-cli-tests parser-tests replay-tool-tests replay-tests gate gate-full ci-local conformance-audit kill-switch-audit stream-purity clean

help:
	echo "Active operator / validation path:"
	echo "  make gate"
	echo "Generic bundle release path:"
	echo "  make bench-check"
	echo "  make release-proof VERSION=1.8.0"
	echo "  make release VERSION=1.8.0"
	echo "  make release-bundle VERSION=1.8.0"
	echo "  make release-bundle-check VERSION=1.8.0"
	echo "  make doc-link-check"
	echo "  make check-workspace"
	echo "  make test"
	echo "  make help-all"

help-all:
	echo "Active operator / validation:"
	echo "  make gate"
	echo "Generic bundle release:"
	echo "  make bench-check"
	echo "  make release-proof VERSION=1.8.0"
	echo "  make release VERSION=1.8.0"
	echo "  make release-bundle VERSION=1.8.0"
	echo "  make release-bundle-check VERSION=1.8.0"
	echo "  make doc-link-check"
	echo "  make check-workspace"
	echo "  make test"
	echo "  make gate-full"
	echo "  make ci-local"
	echo
	echo "Focused grouped help:"
	echo "  make help-demos"
	echo "  make help-firmware"
	echo
	echo "Historical / support validation:"
	echo "  make parser-tests"
	echo "  make replay-tool-tests"
	echo "  make replay-tests"
	echo "  make conformance-audit"
	echo "  make kill-switch-audit"
	echo "  make stream-purity"
	echo
	$(MAKE_NO_PRINT) help-demos
	echo
	$(MAKE_NO_PRINT) help-firmware

help-demos:
	echo "Demo / reference workflows:"
	echo "  Captured divergence:"
	echo "    make capture-demo-A"
	echo "    make capture-demo-B"
	echo "    make demo-captured-verify"
	echo "    make demo-captured-release"
	echo "  Demo V2 lifecycle:"
	echo "    make demo-v2-capture"
	echo "    make demo-v2-verify"
	echo "    make demo-v2-audit-pack"
	echo "    make demo-v2-record"
	echo "  Demo V3 lifecycle:"
	echo "    make demo-v3-verify"
	echo "    make demo-v3-audit-pack"
	echo "    make demo-v3-record"
	echo "    make demo-v3-release"
	echo "  Demo V4 lifecycle:"
	echo "    make demo-v4-verify"
	echo "    make demo-v4-audit-pack"
	echo "    make demo-v4-record"
	echo "    make demo-v4-release"
	echo "  Demo V5 lifecycle:"
	echo "    make demo-v5-verify"
	echo "    make demo-v5-audit-pack"
	echo "    make demo-v5-record"
	echo "    make demo-v5-release"
	echo "  Evidence / explanation:"
	echo "    make demo-evidence-package"
	echo "    make replay-demo-audit"
	echo "  Signal demo:"
	echo "    make demo-signal"
	echo "    make demo-signal-flash"
	echo "    make demo-signal-host-baseline"
	echo "    make demo-signal-host-perturb"
	echo "    make demo-signal-pi-baseline"
	echo "    make demo-signal-pi-perturb"
	echo "    make demo-signal-diff"

help-firmware:
	echo "Experimental / hardware workflows:"
	echo "  Firmware build / flash:"
	echo "    make fw"
	echo "    make fw-bin"
	echo "    make flash"
	echo "    make flash-verify"
	echo "    make flash-compare"
	echo "    make flash-ur"
	echo "    make flash-verify-ur"
	echo "    make flash-compare-ur"
	echo "  Active STM32 capture checks:"
	echo "    make bench-check"
	echo "    make fw-capture-check"
	echo "    make fw-repeat-check"
	echo "    make fw-gate"
	echo "    make firmware-release-check"
	echo "    make fw-release-archive VERSION=<version>"
	echo "  Historical / support replay capture:"
	echo "    make rpl0-replay-check"
	echo "    make rpl0-replay-repeat-check"
	echo "    make rpl0-replay-repeat-auto"
	echo "  Utilities:"
	echo "    make fixture-drift-check"
	echo "    make debug-session"
	echo "    make tim2-smoke"
	echo "    make clean"

fixture-drift-check:
	$(XTASK_WORKFLOW) fixture-drift-check

demo-evidence-package:
	$(XTASK_WORKFLOW) demo-evidence-package

release: release-bundle
	@test -n "$(VERSION)" || { echo "FAIL: VERSION is required. Usage: make release VERSION=<version>"; exit 1; }
	$(MAKE_NO_PRINT) release-bundle-check VERSION="$(VERSION)" > "$(RELEASE_DIR)/make_release_bundle_check.next"
	mv "$(RELEASE_DIR)/make_release_bundle_check.next" "$(RELEASE_DIR)/make_release_bundle_check.txt"
	$(MAKE_NO_PRINT) release-summary VERSION="$(VERSION)"

release-proof:
	@test -n "$(VERSION)" || { echo "FAIL: VERSION is required. Usage: make release-proof VERSION=<version>"; exit 1; }
	if [ "$(RELEASE_PROOF_FIRMWARE)" != "0" ]; then
	  $(PYTHON) scripts/bench_check.py \
	    --serial "$(SERIAL)" \
	    --stflash "$(STFLASH)" \
	    --cargo "$(CARGO)" \
	    --python "$(PYTHON)" \
	    --make "$(MAKE_CMD)" \
	    --fw-elf "$(FW_ELF)" \
	    --fw-bin "$(FW_BIN)" \
	    --firmware-artifacts skip
	fi
	$(MAKE_NO_PRINT) gate
	if [ "$(RELEASE_PROOF_FIRMWARE)" != "0" ]; then
	  $(MAKE_NO_PRINT) fw-gate VERSION="$(VERSION)" SERIAL="$(SERIAL)" STFLASH="$(STFLASH)" FW_GATE_RESET_MODE="$(FW_GATE_RESET_MODE)"
	  $(MAKE_NO_PRINT) fw-release-archive-current VERSION="$(VERSION)" SERIAL="$(SERIAL)"
	fi
	$(MAKE_NO_PRINT) release-bundle VERSION="$(VERSION)"
	$(MAKE_NO_PRINT) release-summary VERSION="$(VERSION)"
	$(MAKE_NO_PRINT) release-bundle-check VERSION="$(VERSION)" > "$(RELEASE_DIR)/make_release_bundle_check.next"
	mv "$(RELEASE_DIR)/make_release_bundle_check.next" "$(RELEASE_DIR)/make_release_bundle_check.txt"
	$(MAKE_NO_PRINT) release-summary VERSION="$(VERSION)"

release-1.7.0:
	$(PYTHON) scripts/release_gate.py \
	  --version 1.7.0 \
	  --release-root "$(RELEASE_ROOT)" \
	  --fw-target "$(FW_TARGET)" \
	  --cargo "$(CARGO)" \
	  --dpw4-pkg "$(DPW4_PKG)" \
	  --make "$(MAKE_CMD)" \
	  --functional \
	  --demo-evidence \
	  --doc-link \
	  --repro \
	  --bundle-check

release-1.8.0:
	$(PYTHON) scripts/release_gate.py \
	  --version 1.8.0 \
	  --release-root "$(RELEASE_ROOT)" \
	  --serial "$(SERIAL)" \
	  --reset-mode "$(FW_GATE_RESET_MODE)" \
	  --fw-target "$(FW_TARGET)" \
	  --cargo "$(CARGO)" \
	  --dpw4-pkg "$(DPW4_PKG)" \
	  --make "$(MAKE_CMD)" \
	  --require-serial \
	  --require-manual-reset \
	  --thumb-check \
	  --functional \
	  --demo-evidence \
	  --replay-tests \
	  --doc-link \
	  --repro \
	  --firmware \
	  --bundle-check

shell-check:
	test -x "$(SHELL)"

stflash-check:
	ST="$$(command -v $(STFLASH) || true)"
	test -n "$$ST" || { echo "FAIL: missing ST-LINK flash binary: $(STFLASH). Install stlink tools or pass STFLASH=/path/to/st-flash."; exit 1; }
	test -x "$$ST" || { echo "FAIL: ST-LINK flash binary is not executable: $$ST"; exit 1; }
	echo "STFLASH=$$ST"

bench-check:
	$(PYTHON) scripts/bench_check.py \
	  --serial "$(SERIAL)" \
	  --stflash "$(STFLASH)" \
	  --cargo "$(CARGO)" \
	  --python "$(PYTHON)" \
	  --make "$(MAKE_CMD)" \
	  --fw-elf "$(FW_ELF)" \
	  --fw-bin "$(FW_BIN)" \
	  --firmware-artifacts "$(BENCH_CHECK_FW_ARTIFACTS)"

fw: shell-check
	$(CARGO) build -p $(FW_PKG) --target $(FW_TARGET) --locked $(FW_FEATURES_ARG)
	$(PYTHON) scripts/check_firmware_elf.py "$(FW_ELF)"

fw-bin: fw
	rm -f "$(FW_BIN)"
	$(CARGO) objcopy -p $(FW_PKG) --target $(FW_TARGET) $(FW_FEATURES_ARG) -- -O binary "$(FW_BIN)"
	test -s "$(FW_BIN)"
	ls -lh "$(FW_BIN)"

flash: fw-bin stflash-check
	$(STFLASH) --reset write "$(FW_BIN)" "$(FLASH_ADDR)"

flash-ur: fw-bin stflash-check
	$(STFLASH) --connect-under-reset --freq=200K --reset write "$(FW_BIN)" "$(FLASH_ADDR)"

flash-verify: fw-bin stflash-check
	rm -f "$(FLASH_HEAD)"
	$(STFLASH) read "$(FLASH_HEAD)" "$(FLASH_ADDR)" 64
	test -s "$(FLASH_HEAD)"
	hexdump -C "$(FLASH_HEAD)" | sed -n '1,4p'
	$(PYTHON) scripts/check_flash_vectors.py "$(FLASH_HEAD)"

flash-verify-ur: fw-bin stflash-check
	rm -f "$(FLASH_HEAD)"
	$(STFLASH) --connect-under-reset --freq=200K read "$(FLASH_HEAD)" "$(FLASH_ADDR)" 64
	test -s "$(FLASH_HEAD)"
	hexdump -C "$(FLASH_HEAD)" | sed -n '1,4p'
	$(PYTHON) scripts/check_flash_vectors.py "$(FLASH_HEAD)"

flash-compare: fw-bin stflash-check
	bash scripts/compare_flash_image.sh --label flash-compare --stflash "$(STFLASH)" --addr "$(FLASH_ADDR)" --image "$(FW_BIN)" --out "$(FLASH_FULL)"

flash-compare-ur: fw-bin stflash-check
	bash scripts/compare_flash_image.sh --label flash-compare-ur --stflash "$(STFLASH)" --addr "$(FLASH_ADDR)" --image "$(FW_BIN)" --out "$(FLASH_FULL)" --under-reset

demo-signal:
	echo "Signal demo runs on two machines."
	echo "1. host: make demo-signal-flash"
	echo "2. host: make demo-signal-host-baseline"
	echo "3. pi:   make demo-signal-pi-baseline"
	echo "4. host: make demo-signal-host-perturb"
	echo "5. pi:   make demo-signal-pi-perturb"
	echo "6. host: make demo-signal-diff"

demo-signal-flash:
	$(MAKE) fw-bin
	$(MAKE) flash-ur SERIAL="$(SERIAL)"

demo-signal-host-baseline:
	rm -f "$(SIGNAL_BASELINE_CSV)"
	echo "Starting host capture for baseline on $(SERIAL)."
	echo "Next steps:"
	echo "  1. Listener is starting now."
	echo "  2. Reset STM32 after the listener is attached."
	echo "  3. Run on the Pi: make demo-signal-pi-baseline"
	$(PYTHON) scripts/csv_capture.py --serial "$(SERIAL)" --out "$(SIGNAL_BASELINE_CSV)"

demo-signal-host-perturb:
	rm -f "$(SIGNAL_OBSERVED_CSV)"
	echo "Starting host capture for perturbation on $(SERIAL)."
	echo "Next steps:"
	echo "  1. Listener is starting now."
	echo "  2. Reset STM32 after the listener is attached."
	echo "  3. Run on the Pi: make demo-signal-pi-perturb"
	$(PYTHON) scripts/csv_capture.py --serial "$(SERIAL)" --out "$(SIGNAL_OBSERVED_CSV)"

demo-signal-pi-baseline:
	command -v python3 >/dev/null || { echo "python3 is required for the Pi emitter"; exit 1; }
	python3 -c 'import gpiod' >/dev/null 2>&1 || { \
		echo "WARNING: gpiod not available; Pi emitter not runnable in this environment"; \
		echo "This is expected outside a Raspberry Pi runtime"; \
		exit 0; \
	}
	test -e /dev/gpiochip0 || { \
		echo "WARNING: /dev/gpiochip0 not present; Pi emitter requires real GPIO hardware"; \
		exit 0; \
	}
	echo "Pi emitter launch: mode=baseline gpio=GPIO17 frames=$(SIGNAL_FRAMES)"
	echo "Runtime: python3 + gpiod + GPIO character device access; no pigpio/pigpiod"
	$(PYTHON) scripts/pi_emitter.py --mode baseline --frames "$(SIGNAL_FRAMES)" --perturb-frame "$(SIGNAL_PERTURB_FRAME)" $(if $(SIGNAL_REPEAT_SECONDS),--repeat-seconds "$(SIGNAL_REPEAT_SECONDS)")

demo-signal-pi-perturb:
	command -v python3 >/dev/null || { echo "python3 is required for the Pi emitter"; exit 1; }
	python3 -c 'import gpiod' >/dev/null 2>&1 || { \
		echo "WARNING: gpiod not available; Pi emitter not runnable in this environment"; \
		echo "This is expected outside a Raspberry Pi runtime"; \
		exit 0; \
	}
	test -e /dev/gpiochip0 || { \
		echo "WARNING: /dev/gpiochip0 not present; Pi emitter requires real GPIO hardware"; \
		exit 0; \
	}
	echo "Pi emitter launch: mode=perturb gpio=GPIO17 frames=$(SIGNAL_FRAMES) perturb_frame=$(SIGNAL_PERTURB_FRAME)"
	echo "Runtime: python3 + gpiod + GPIO character device access; no pigpio/pigpiod"
	$(PYTHON) scripts/pi_emitter.py --mode perturb --frames "$(SIGNAL_FRAMES)" --perturb-frame "$(SIGNAL_PERTURB_FRAME)" $(if $(SIGNAL_REPEAT_SECONDS),--repeat-seconds "$(SIGNAL_REPEAT_SECONDS)")

demo-signal-diff:
	$(PYTHON) scripts/interval_diff.py "$(SIGNAL_BASELINE_CSV)" "$(SIGNAL_OBSERVED_CSV)"

# Timing-crate interval CSV path (replay-fw-f446-timing).
# Pruned from fw-gate; retained for direct timing-crate operator use.
# capture waits for STATE,CAPTURE_DONE,138 and validates index,interval_us CSV via replay-host.
fw-capture-check:
	test -n "$(SERIAL)" || { echo "SERIAL not set"; exit 1; }
	rm -rf "$(FW_CAPTURE_DIR)"
	SERIAL="$(SERIAL)" $(PYTHON) scripts/replay_repeat_check.py \
	  --runs 1 \
	  --reset-mode "$(FW_GATE_RESET_MODE)" \
	  --timeout "$(FW_CAPTURE_TIMEOUT)" \
	  --artifacts-dir "$(FW_CAPTURE_DIR)"

fw-repeat-check:
	test -n "$(SERIAL)" || { echo "SERIAL not set"; exit 1; }
	rm -rf "$(FW_REPEAT_DIR)"
	SERIAL="$(SERIAL)" $(PYTHON) scripts/replay_repeat_check.py \
	  --runs "$(REPLAY_REPEAT_RUNS)" \
	  --reset-mode "$(FW_GATE_RESET_MODE)" \
	  --timeout "$(FW_CAPTURE_TIMEOUT)" \
	  --artifacts-dir "$(FW_REPEAT_DIR)"

# Active RPL0 operator path (replay-fw-f446).
# capture waits for replay header after board boots post-flash.
rpl0-replay-check:
	export PYTHONPATH="$(CURDIR)$${PYTHONPATH:+:$${PYTHONPATH}}"
	$(MAKE) flash-ur
	SERIAL="$(SERIAL)" $(ARTIFACT_TOOL) capture --quick --reset-context stlink --out "$(REPLAY_RUN)"
	$(ARTIFACT_TOOL) verify "$(REPLAY_RUN)" --signal-model "$(REPLAY_SIGNAL_MODEL)"
	$(ARTIFACT_TOOL) compare "$(REPLAY_BASELINE)" "$(REPLAY_RUN)"

rpl0-replay-repeat-check:
	export PYTHONPATH="$(CURDIR)$${PYTHONPATH:+:$${PYTHONPATH}}"
	SERIAL="$(SERIAL)" $(PYTHON) scripts/repeat_capture.py \
	  --contract rpl0 \
	  --runs "$(REPLAY_REPEAT_RUNS)" \
	  --signal-model "$(REPLAY_SIGNAL_MODEL)" \
	  --artifacts-dir "$(REPLAY_REPEAT_DIR)"

rpl0-replay-repeat-auto: stflash-check
	export PYTHONPATH="$(CURDIR)$${PYTHONPATH:+:$${PYTHONPATH}}"
	SERIAL="$(SERIAL)" $(PYTHON) scripts/repeat_capture.py \
	  --contract rpl0 \
	  --runs "$(REPLAY_REPEAT_RUNS)" \
	  --signal-model "$(REPLAY_SIGNAL_MODEL)" \
	  --reset-mode stlink \
	  --stflash "$(STFLASH)" \
	  --artifacts-dir "$(REPLAY_REPEAT_DIR)"

fw-gate:
	@test -n "$(SERIAL)" || { echo "FAIL: SERIAL is required. Pass SERIAL=/dev/ttyACM<N> for the STM32 ST-LINK VCP."; exit 1; }
	$(PYTHON) scripts/fw_gate.py \
	  --serial "$(SERIAL)" \
	  --reset-mode "$(FW_GATE_RESET_MODE)" \
	  --capture-timeout "$(REPLAY_CAPTURE_TIMEOUT)" \
	  --repeat-runs "$(REPLAY_REPEAT_RUNS)" \
	  --signal-model "$(REPLAY_SIGNAL_MODEL)" \
	  --replay-run "$(REPLAY_RUN)" \
	  --replay-baseline "$(REPLAY_BASELINE)" \
	  --repeat-dir "$(REPLAY_REPEAT_DIR)" \
	  --stflash "$(STFLASH)" \
	  --make "$(MAKE_CMD)"

firmware-release-summary:
	@echo "REPLAY_RUN=$(REPLAY_RUN)"; \
	echo "REPLAY_REPEAT_DIR=$(REPLAY_REPEAT_DIR)"; \
	echo; \
	echo "== capture file =="; \
	ls -lah "$(REPLAY_RUN)"; \
	sha256sum "$(REPLAY_RUN)"; \
	echo; \
	echo "== repeat files =="; \
	ls -lah "$(REPLAY_REPEAT_DIR)"; \
	echo; \
	echo "== repeat manifest =="; \
	cat "$(REPLAY_REPEAT_DIR)/replay_manifest_v1.txt"

firmware-release-check: fw-gate firmware-release-summary

fw-release-archive-current: firmware-release-summary
	@test -n "$(SERIAL)" || { echo "FAIL: SERIAL is required. Pass SERIAL=/dev/ttyACM<N> for the STM32 ST-LINK VCP."; exit 1; }
	@test -n "$(VERSION)" || { echo "FAIL: VERSION is required. Usage: make fw-release-archive-current VERSION=<version>"; exit 1; }
	$(PYTHON) scripts/archive_firmware_evidence.py \
	  --version "$(VERSION)" \
	  --release-root "$(RELEASE_ROOT)" \
	  --replay-run "$(REPLAY_RUN)" \
	  --repeat-dir "$(REPLAY_REPEAT_DIR)"

fw-release-archive: firmware-release-check
	$(MAKE) fw-release-archive-current VERSION="$(VERSION)" SERIAL="$(SERIAL)"

release-bundle:
	@test -n "$(VERSION)" || { echo "FAIL: VERSION is required. Usage: make release-bundle VERSION=<version>"; exit 1; }
	$(PYTHON) scripts/release_bundle.py \
	  --version "$(VERSION)" \
	  --release-root "$(RELEASE_ROOT)" \
	  --fw-target "$(FW_TARGET)" \
	  --cargo "$(CARGO)" \
	  --dpw4-pkg "$(DPW4_PKG)" \
	  --make "$(MAKE_CMD)"

release-summary:
	@test -n "$(VERSION)" || { echo "FAIL: VERSION is required. Usage: make release-summary VERSION=<version>"; exit 1; }
	$(PYTHON) scripts/release_summary.py \
	  --version "$(VERSION)" \
	  --release-root "$(RELEASE_ROOT)"

release-bundle-check:
	@test -n "$(VERSION)" || { echo "FAIL: VERSION is required. Usage: make release-bundle-check VERSION=<version>"; exit 1; }
	$(PYTHON) scripts/check_release_bundle.py --version "$(VERSION)"

capture-demo-A: shell-check stflash-check
	echo "== Captured Divergence Demo: Run A (canonical firmware) =="
	mkdir -p "$(DEMO_CAPTURED_A_DIR)"
	test -n "$(SERIAL)" || { echo "SERIAL not set"; exit 1; }
	$(MAKE) FW_FEATURES= flash-ur
	SERIAL="$(SERIAL)" $(PYTHON) scripts/repeat_capture.py \
	  --contract rpl0 \
	  --runs 1 \
	  --signal-model "$(REPLAY_SIGNAL_MODEL)" \
	  --reset-mode stlink \
	  --stflash "$(STFLASH)" \
	  --artifacts-dir "$(DEMO_CAPTURED_A_DIR)"
	test -s "$(DEMO_CAPTURED_A_TMP)"
	cp "$(DEMO_CAPTURED_A_TMP)" "$(DEMO_CAPTURED_A)"
	echo "Wrote $(DEMO_CAPTURED_A)"

capture-demo-B: shell-check stflash-check
	echo "== Captured Divergence Demo: Run B (demo-divergence firmware) =="
	mkdir -p "$(DEMO_CAPTURED_B_DIR)"
	test -n "$(SERIAL)" || { echo "SERIAL not set"; exit 1; }
	$(MAKE) FW_FEATURES=demo-divergence flash-ur
	SERIAL="$(SERIAL)" $(PYTHON) scripts/repeat_capture.py \
	  --contract rpl0 \
	  --runs 1 \
	  --signal-model "$(REPLAY_SIGNAL_MODEL)" \
	  --reset-mode stlink \
	  --stflash "$(STFLASH)" \
	  --artifacts-dir "$(DEMO_CAPTURED_B_DIR)"
	test -s "$(DEMO_CAPTURED_B_TMP)"
	cp "$(DEMO_CAPTURED_B_TMP)" "$(DEMO_CAPTURED_B)"
	echo "Wrote $(DEMO_CAPTURED_B)"

demo-captured-verify: shell-check
	test -s "$(DEMO_CAPTURED_A)"
	test -s "$(DEMO_CAPTURED_B)"
	if cmp -s "$(DEMO_CAPTURED_A)" "$(DEMO_CAPTURED_B)"; then \
	  echo "ERROR: expected captured divergence artifacts to differ"; \
	  exit 1; \
	fi
	$(PYTHON) scripts/artifact_diff.py "$(DEMO_CAPTURED_A)" "$(DEMO_CAPTURED_B)"

demo-captured-release: demo-captured-verify
	echo "Captured divergence evidence verified."

demo-v2-capture: shell-check stflash-check
	echo "== Demo V2 Capture: Canonical =="
	mkdir -p "$(DEMO_V2_CAPTURE_A_DIR)" "$(DEMO_V2_CAPTURE_B_DIR)"
	$(MAKE) FW_FEATURES= flash-ur
	SERIAL="$(SERIAL)" $(PYTHON) scripts/repeat_capture.py \
	  --contract rpl0 \
	  --runs 1 \
	  --signal-model "$(REPLAY_SIGNAL_MODEL)" \
	  --reset-mode stlink \
	  --stflash "$(STFLASH)" \
	  --artifacts-dir "$(DEMO_V2_CAPTURE_A_DIR)"
	test -s "$(DEMO_V2_CAPTURE_A_TMP)"
	cp "$(DEMO_V2_CAPTURE_A_TMP)" "$(DEMO_V2_CAPTURED_A)"
	echo "Wrote $(DEMO_V2_CAPTURED_A)"
	echo "== Demo V2 Capture: Perturbed (demo-persistent-divergence) =="
	$(MAKE) FW_FEATURES=demo-persistent-divergence flash-ur
	SERIAL="$(SERIAL)" $(PYTHON) scripts/repeat_capture.py \
	  --contract rpl0 \
	  --runs 1 \
	  --signal-model "$(REPLAY_SIGNAL_MODEL)" \
	  --reset-mode stlink \
	  --stflash "$(STFLASH)" \
	  --artifacts-dir "$(DEMO_V2_CAPTURE_B_DIR)"
	test -s "$(DEMO_V2_CAPTURE_B_TMP)"
	cp "$(DEMO_V2_CAPTURE_B_TMP)" "$(DEMO_V2_CAPTURED_B)"
	echo "Wrote $(DEMO_V2_CAPTURED_B)"

demo-v2-fixture-verify: shell-check
	test -s "$(DEMO_V2_FIXTURE_A)"
	test -s "$(DEMO_V2_FIXTURE_B)"
	echo "== Demo V2 Verify: Fixture =="
	$(PYTHON) scripts/artifact_diff.py "$(DEMO_V2_FIXTURE_A)" "$(DEMO_V2_FIXTURE_B)"
	$(ARTIFACT_TOOL) inspect "$(DEMO_V2_FIXTURE_A)" --frames 4093:4100
	$(ARTIFACT_TOOL) inspect "$(DEMO_V2_FIXTURE_B)" --frames 4093:4100
	$(ARTIFACT_TOOL) verify "$(DEMO_V2_FIXTURE_A)" --signal-model "$(REPLAY_SIGNAL_MODEL)"
	if $(ARTIFACT_TOOL) verify "$(DEMO_V2_FIXTURE_B)" --signal-model "$(REPLAY_SIGNAL_MODEL)"; then \
	  echo "ERROR: expected $(REPLAY_SIGNAL_MODEL) verification failure for $(DEMO_V2_FIXTURE_B)"; \
	  exit 1; \
	else \
	  echo "Expected failure observed: $(DEMO_V2_FIXTURE_B) with signal-model $(REPLAY_SIGNAL_MODEL)"; \
	fi
	$(ARTIFACT_TOOL) verify "$(DEMO_V2_FIXTURE_B)" --signal-model none

demo-v2-verify: demo-v2-fixture-verify
	echo "== Demo V2 Verify: Captured =="
	if [[ ! -s "$(DEMO_V2_CAPTURED_A)" || ! -s "$(DEMO_V2_CAPTURED_B)" ]]; then \
	  echo "Captured artifacts missing; run 'make demo-v2-capture' to generate $(DEMO_V2_CAPTURED_A) and $(DEMO_V2_CAPTURED_B)"; \
	  exit 1; \
	fi
	$(PYTHON) scripts/artifact_diff.py "$(DEMO_V2_CAPTURED_A)" "$(DEMO_V2_CAPTURED_B)"
	$(ARTIFACT_TOOL) inspect "$(DEMO_V2_CAPTURED_A)" --frames 4093:4100
	$(ARTIFACT_TOOL) inspect "$(DEMO_V2_CAPTURED_B)" --frames 4093:4100
	$(ARTIFACT_TOOL) verify "$(DEMO_V2_CAPTURED_A)" --signal-model "$(REPLAY_SIGNAL_MODEL)"
	if $(ARTIFACT_TOOL) verify "$(DEMO_V2_CAPTURED_B)" --signal-model "$(REPLAY_SIGNAL_MODEL)"; then \
	  echo "ERROR: expected $(REPLAY_SIGNAL_MODEL) verification failure for $(DEMO_V2_CAPTURED_B)"; \
	  exit 1; \
	else \
	  echo "Expected failure observed: $(DEMO_V2_CAPTURED_B) with signal-model $(REPLAY_SIGNAL_MODEL)"; \
	fi
	$(ARTIFACT_TOOL) verify "$(DEMO_V2_CAPTURED_B)" --signal-model none

demo-v2-audit-pack: shell-check
	echo "== Demo V2 Audit Pack =="
	echo "Git"
	if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then \
	  echo "commit=$$(git rev-parse HEAD)"; \
	else \
	  echo "commit=unavailable"; \
	fi
	echo
	echo "Fixture Artifacts"
	echo "$(DEMO_V2_FIXTURE_A)"
	echo "$(DEMO_V2_FIXTURE_B)"
	test -s "$(DEMO_V2_FIXTURE_A)"
	test -s "$(DEMO_V2_FIXTURE_B)"
	$(ARTIFACT_TOOL) hash "$(DEMO_V2_FIXTURE_A)"
	$(ARTIFACT_TOOL) hash "$(DEMO_V2_FIXTURE_B)"
	echo
	echo "Captured Artifacts"
	if [[ -s "$(DEMO_V2_CAPTURED_A)" && -s "$(DEMO_V2_CAPTURED_B)" ]]; then \
	  echo "$(DEMO_V2_CAPTURED_A)"; \
	  echo "$(DEMO_V2_CAPTURED_B)"; \
	  $(ARTIFACT_TOOL) hash "$(DEMO_V2_CAPTURED_A)"; \
	  $(ARTIFACT_TOOL) hash "$(DEMO_V2_CAPTURED_B)"; \
	  CAPTURED_PRESENT=1; \
	else \
	  echo "captured artifacts absent (run 'make demo-v2-capture')"; \
	  CAPTURED_PRESENT=0; \
	fi; \
	echo; \
	echo "Fixture Diff"; \
	$(PYTHON) scripts/artifact_diff.py "$(DEMO_V2_FIXTURE_A)" "$(DEMO_V2_FIXTURE_B)"; \
	echo; \
	echo "Fixture Inspect A"; \
	$(ARTIFACT_TOOL) inspect "$(DEMO_V2_FIXTURE_A)" --frames 4093:4100; \
	echo; \
	echo "Fixture Inspect B"; \
	$(ARTIFACT_TOOL) inspect "$(DEMO_V2_FIXTURE_B)" --frames 4093:4100; \
	echo; \
	echo "Fixture Verify"; \
	$(ARTIFACT_TOOL) verify "$(DEMO_V2_FIXTURE_A)" --signal-model "$(REPLAY_SIGNAL_MODEL)"; \
	if $(ARTIFACT_TOOL) verify "$(DEMO_V2_FIXTURE_B)" --signal-model "$(REPLAY_SIGNAL_MODEL)"; then \
	  echo "ERROR: expected $(REPLAY_SIGNAL_MODEL) verification failure for $(DEMO_V2_FIXTURE_B)"; \
	  exit 1; \
	else \
	  echo "Expected failure observed: $(DEMO_V2_FIXTURE_B) with signal-model $(REPLAY_SIGNAL_MODEL)"; \
	fi; \
	$(ARTIFACT_TOOL) verify "$(DEMO_V2_FIXTURE_B)" --signal-model none; \
	if [[ "$$CAPTURED_PRESENT" -eq 1 ]]; then \
	  echo; \
	  echo "Captured Diff"; \
	  $(PYTHON) scripts/artifact_diff.py "$(DEMO_V2_CAPTURED_A)" "$(DEMO_V2_CAPTURED_B)"; \
	  echo; \
	  echo "Captured Inspect A"; \
	  $(ARTIFACT_TOOL) inspect "$(DEMO_V2_CAPTURED_A)" --frames 4093:4100; \
	  echo; \
	  echo "Captured Inspect B"; \
	  $(ARTIFACT_TOOL) inspect "$(DEMO_V2_CAPTURED_B)" --frames 4093:4100; \
	  echo; \
	  echo "Captured Verify"; \
	  $(ARTIFACT_TOOL) verify "$(DEMO_V2_CAPTURED_A)" --signal-model "$(REPLAY_SIGNAL_MODEL)"; \
	  if $(ARTIFACT_TOOL) verify "$(DEMO_V2_CAPTURED_B)" --signal-model "$(REPLAY_SIGNAL_MODEL)"; then \
	    echo "ERROR: expected $(REPLAY_SIGNAL_MODEL) verification failure for $(DEMO_V2_CAPTURED_B)"; \
	    exit 1; \
	  else \
	    echo "Expected failure observed: $(DEMO_V2_CAPTURED_B) with signal-model $(REPLAY_SIGNAL_MODEL)"; \
	  fi; \
	  $(ARTIFACT_TOOL) verify "$(DEMO_V2_CAPTURED_B)" --signal-model none; \
	else \
	  echo; \
	  echo "Captured Diff"; \
	  echo "captured artifacts absent (run 'make demo-v2-capture')"; \
	  echo; \
	  echo "Captured Inspect A"; \
	  echo "captured artifacts absent (run 'make demo-v2-capture')"; \
	  echo; \
	  echo "Captured Inspect B"; \
	  echo "captured artifacts absent (run 'make demo-v2-capture')"; \
	  echo; \
	  echo "Captured Verify"; \
	  echo "captured artifacts absent (run 'make demo-v2-capture')"; \
	fi

demo-v2-record: shell-check
	mkdir -p "$(DEMO_V2_RUNLOG_DIR)"
	SHORT_SHA="$$(git rev-parse --short HEAD 2>/dev/null || echo unknown)"
	FULL_SHA="$$(git rev-parse HEAD 2>/dev/null || echo unavailable)"
	UTC_DATE="$$(date -u '+%Y-%m-%dT%H:%M:%SZ')"
	OUT_FILE="$(DEMO_V2_RUNLOG_DIR)/demo_v2_release_$${SHORT_SHA}.txt"
	{
	  echo "== Demo V2 Verification Record =="
	  echo "commit=$${FULL_SHA}"
	  echo "date=$${UTC_DATE}"
	  echo
	  echo "== Verification =="
	  $(MAKE_NO_PRINT) demo-v2-verify
	  echo
	  echo "== Audit Pack =="
	  $(MAKE_NO_PRINT) demo-v2-audit-pack
	} 2>&1 | tee "$${OUT_FILE}"
	echo "Wrote $${OUT_FILE}"

demo-v3-verify: shell-check
	$(PYTHON) scripts/generate_demo_v3_fixtures.py --out-dir "$(DEMO_V3_DIR)"
	test -s "$(DEMO_V3_TRANSIENT_A)"
	test -s "$(DEMO_V3_TRANSIENT_B)"
	test -s "$(DEMO_V3_OFFSET_A)"
	test -s "$(DEMO_V3_OFFSET_B)"
	test -s "$(DEMO_V3_RATE_A)"
	test -s "$(DEMO_V3_RATE_B)"
	check_classification() { \
	  local artifact_a="$$1"; \
	  local artifact_b="$$2"; \
	  local expected="$$3"; \
	  local expected_frame="$$4"; \
	  local out; \
	  out="$$($(PYTHON) scripts/artifact_diff.py "$$artifact_a" "$$artifact_b")"; \
	  echo "$$out"; \
	  echo "$$out" | grep -q "First divergence frame: $$expected_frame" || { \
	    echo "ERROR: expected first divergence frame '$$expected_frame' for $$artifact_a vs $$artifact_b"; \
	    exit 1; \
	  }; \
	  echo "$$out" | grep -q "Classification: $$expected" || { \
	    echo "ERROR: expected classification '$$expected' for $$artifact_a vs $$artifact_b"; \
	    exit 1; \
	  }; \
	}
	echo "== Demo V3 Verify: transient =="
	check_classification "$(DEMO_V3_TRANSIENT_A)" "$(DEMO_V3_TRANSIENT_B)" transient 4096
	$(ARTIFACT_TOOL) inspect "$(DEMO_V3_TRANSIENT_A)" --frames 4093:4103
	$(ARTIFACT_TOOL) inspect "$(DEMO_V3_TRANSIENT_B)" --frames 4093:4103
	echo "== Demo V3 Verify: persistent_offset =="
	check_classification "$(DEMO_V3_OFFSET_A)" "$(DEMO_V3_OFFSET_B)" persistent_offset 4096
	$(ARTIFACT_TOOL) inspect "$(DEMO_V3_OFFSET_A)" --frames 4093:4103
	$(ARTIFACT_TOOL) inspect "$(DEMO_V3_OFFSET_B)" --frames 4093:4103
	echo "== Demo V3 Verify: rate_divergence =="
	check_classification "$(DEMO_V3_RATE_A)" "$(DEMO_V3_RATE_B)" rate_divergence 4096
	$(ARTIFACT_TOOL) inspect "$(DEMO_V3_RATE_A)" --frames 4093:4103
	$(ARTIFACT_TOOL) inspect "$(DEMO_V3_RATE_B)" --frames 4093:4103

demo-v3-audit-pack: shell-check
	$(PYTHON) scripts/generate_demo_v3_fixtures.py --out-dir "$(DEMO_V3_DIR)"
	echo "== Demo V3 Audit Pack =="
	echo "Git"
	if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then \
	  echo "commit=$$(git rev-parse HEAD)"; \
	else \
	  echo "commit=unavailable"; \
	fi
	echo
	echo "Fixtures"
	for p in "$(DEMO_V3_TRANSIENT_A)" "$(DEMO_V3_TRANSIENT_B)" "$(DEMO_V3_OFFSET_A)" "$(DEMO_V3_OFFSET_B)" "$(DEMO_V3_RATE_A)" "$(DEMO_V3_RATE_B)"; do \
	  test -s "$$p"; \
	  echo "$$p"; \
	  $(ARTIFACT_TOOL) hash "$$p"; \
	done
	echo
	echo "Transient Diff"
	$(PYTHON) scripts/artifact_diff.py "$(DEMO_V3_TRANSIENT_A)" "$(DEMO_V3_TRANSIENT_B)"
	echo
	echo "Transient Inspect A"
	$(ARTIFACT_TOOL) inspect "$(DEMO_V3_TRANSIENT_A)" --frames 4093:4103
	echo
	echo "Transient Inspect B"
	$(ARTIFACT_TOOL) inspect "$(DEMO_V3_TRANSIENT_B)" --frames 4093:4103
	echo
	echo "Persistent Offset Diff"
	$(PYTHON) scripts/artifact_diff.py "$(DEMO_V3_OFFSET_A)" "$(DEMO_V3_OFFSET_B)"
	echo
	echo "Persistent Offset Inspect A"
	$(ARTIFACT_TOOL) inspect "$(DEMO_V3_OFFSET_A)" --frames 4093:4103
	echo
	echo "Persistent Offset Inspect B"
	$(ARTIFACT_TOOL) inspect "$(DEMO_V3_OFFSET_B)" --frames 4093:4103
	echo
	echo "Rate Divergence Diff"
	$(PYTHON) scripts/artifact_diff.py "$(DEMO_V3_RATE_A)" "$(DEMO_V3_RATE_B)"
	echo
	echo "Rate Divergence Inspect A"
	$(ARTIFACT_TOOL) inspect "$(DEMO_V3_RATE_A)" --frames 4093:4103
	echo
	echo "Rate Divergence Inspect B"
	$(ARTIFACT_TOOL) inspect "$(DEMO_V3_RATE_B)" --frames 4093:4103

demo-v3-record: shell-check
	mkdir -p "$(DEMO_V3_RUNLOG_DIR)"
	SHORT_SHA="$$(git rev-parse --short HEAD 2>/dev/null || echo unknown)"
	FULL_SHA="$$(git rev-parse HEAD 2>/dev/null || echo unavailable)"
	UTC_DATE="$$(date -u '+%Y-%m-%dT%H:%M:%SZ')"
	OUT_FILE="$(DEMO_V3_RUNLOG_DIR)/demo_v3_release_$${SHORT_SHA}.txt"
	{
	  echo "== Demo V3 Verification Record =="
	  echo "commit=$${FULL_SHA}"
	  echo "date=$${UTC_DATE}"
	  echo
	  echo "== Verification =="
	  $(MAKE_NO_PRINT) demo-v3-verify
	  echo
	  echo "== Audit Pack =="
	  $(MAKE_NO_PRINT) demo-v3-audit-pack
	} 2>&1 | tee "$${OUT_FILE}"
	echo "Wrote $${OUT_FILE}"

demo-v3-release: shell-check
	$(MAKE_NO_PRINT) demo-v3-verify
	$(MAKE_NO_PRINT) demo-v3-audit-pack
	$(MAKE_NO_PRINT) demo-v3-record

demo-v4-verify: shell-check
	$(PYTHON) scripts/generate_demo_v4_fixtures.py --out-dir "$(DEMO_V4_DIR)"
	for p in "$(DEMO_V4_HEADER_A)" "$(DEMO_V4_HEADER_B)" "$(DEMO_V4_HEADER_SAMPLE_A)" "$(DEMO_V4_HEADER_SAMPLE_B)" "$(DEMO_V4_TIMER_A)" "$(DEMO_V4_TIMER_B)" "$(DEMO_V4_IRQ_A)" "$(DEMO_V4_IRQ_B)" "$(DEMO_V4_SAMPLE_A)" "$(DEMO_V4_SAMPLE_B)" "$(DEMO_V4_MIXED_A)" "$(DEMO_V4_MIXED_B)"; do \
	  test -s "$$p"; \
	done
	python3 tests/test_demo_v4_region_attribution.py
	echo "== Demo V4 Verify: header_schema =="
	$(PYTHON) scripts/artifact_diff.py "$(DEMO_V4_HEADER_A)" "$(DEMO_V4_HEADER_B)"
	echo
	echo "== Demo V4 Verify: header_schema + sample_payload @ frame 0 =="
	$(PYTHON) scripts/artifact_diff.py "$(DEMO_V4_HEADER_SAMPLE_A)" "$(DEMO_V4_HEADER_SAMPLE_B)"
	echo
	echo "== Demo V4 Verify: timer_delta =="
	$(PYTHON) scripts/artifact_diff.py "$(DEMO_V4_TIMER_A)" "$(DEMO_V4_TIMER_B)"
	$(ARTIFACT_TOOL) inspect "$(DEMO_V4_TIMER_A)" --frames 4093:4103
	$(ARTIFACT_TOOL) inspect "$(DEMO_V4_TIMER_B)" --frames 4093:4103
	echo
	echo "== Demo V4 Verify: irq_state =="
	$(PYTHON) scripts/artifact_diff.py "$(DEMO_V4_IRQ_A)" "$(DEMO_V4_IRQ_B)"
	$(ARTIFACT_TOOL) inspect "$(DEMO_V4_IRQ_A)" --frames 4093:4103
	$(ARTIFACT_TOOL) inspect "$(DEMO_V4_IRQ_B)" --frames 4093:4103
	echo
	echo "== Demo V4 Verify: sample_payload =="
	$(PYTHON) scripts/artifact_diff.py "$(DEMO_V4_SAMPLE_A)" "$(DEMO_V4_SAMPLE_B)"
	$(ARTIFACT_TOOL) inspect "$(DEMO_V4_SAMPLE_A)" --frames 4093:4103
	$(ARTIFACT_TOOL) inspect "$(DEMO_V4_SAMPLE_B)" --frames 4093:4103
	echo
	echo "== Demo V4 Verify: mixed =="
	$(PYTHON) scripts/artifact_diff.py "$(DEMO_V4_MIXED_A)" "$(DEMO_V4_MIXED_B)"
	$(ARTIFACT_TOOL) inspect "$(DEMO_V4_MIXED_A)" --frames 4093:4103
	$(ARTIFACT_TOOL) inspect "$(DEMO_V4_MIXED_B)" --frames 4093:4103

demo-v4-audit-pack: shell-check
	$(PYTHON) scripts/generate_demo_v4_fixtures.py --out-dir "$(DEMO_V4_DIR)"
	echo "== Demo V4 Audit Pack =="
	echo "Git"
	if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then \
	  echo "commit=$$(git rev-parse HEAD)"; \
	else \
	  echo "commit=unavailable"; \
	fi
	echo
	echo "Fixtures"
	for p in "$(DEMO_V4_HEADER_A)" "$(DEMO_V4_HEADER_B)" "$(DEMO_V4_HEADER_SAMPLE_A)" "$(DEMO_V4_HEADER_SAMPLE_B)" "$(DEMO_V4_TIMER_A)" "$(DEMO_V4_TIMER_B)" "$(DEMO_V4_IRQ_A)" "$(DEMO_V4_IRQ_B)" "$(DEMO_V4_SAMPLE_A)" "$(DEMO_V4_SAMPLE_B)" "$(DEMO_V4_MIXED_A)" "$(DEMO_V4_MIXED_B)"; do \
	  test -s "$$p"; \
	  echo "$$p"; \
	  $(ARTIFACT_TOOL) hash "$$p"; \
	done
	echo
	echo "Header/Schema Diff"
	$(PYTHON) scripts/artifact_diff.py "$(DEMO_V4_HEADER_A)" "$(DEMO_V4_HEADER_B)"
	echo
	echo "Header/Schema + Sample Payload @ Frame 0 Diff"
	$(PYTHON) scripts/artifact_diff.py "$(DEMO_V4_HEADER_SAMPLE_A)" "$(DEMO_V4_HEADER_SAMPLE_B)"
	echo
	echo "Timer Delta Diff"
	$(PYTHON) scripts/artifact_diff.py "$(DEMO_V4_TIMER_A)" "$(DEMO_V4_TIMER_B)"
	echo
	echo "IRQ State Diff"
	$(PYTHON) scripts/artifact_diff.py "$(DEMO_V4_IRQ_A)" "$(DEMO_V4_IRQ_B)"
	echo
	echo "Sample Payload Diff"
	$(PYTHON) scripts/artifact_diff.py "$(DEMO_V4_SAMPLE_A)" "$(DEMO_V4_SAMPLE_B)"
	echo
	echo "Mixed Diff"
	$(PYTHON) scripts/artifact_diff.py "$(DEMO_V4_MIXED_A)" "$(DEMO_V4_MIXED_B)"

demo-v4-record: shell-check
	{ \
	  mkdir -p "$(DEMO_V4_RUNLOG_DIR)"; \
	  SHORT_SHA="$$(git rev-parse --short HEAD 2>/dev/null || echo unknown)"; \
	  FULL_SHA="$$(git rev-parse HEAD 2>/dev/null || echo unavailable)"; \
	  UTC_DATE="$$(date -u '+%Y-%m-%dT%H:%M:%SZ')"; \
	  OUT_FILE="$(DEMO_V4_RUNLOG_DIR)/demo_v4_release_$${SHORT_SHA}.txt"; \
	  { \
	    echo "== Demo V4 Verification Record =="; \
	    echo "commit=$${FULL_SHA}"; \
	    echo "date=$${UTC_DATE}"; \
	    echo; \
	    echo "== Verification =="; \
	    $(MAKE_NO_PRINT) demo-v4-verify; \
	    echo; \
	    echo "== Audit Pack =="; \
	    $(MAKE_NO_PRINT) demo-v4-audit-pack; \
	  } 2>&1 | tee "$${OUT_FILE}"; \
	  echo "Wrote $${OUT_FILE}"; \
	}

demo-v4-release: shell-check
	$(MAKE_NO_PRINT) demo-v4-verify
	$(MAKE_NO_PRINT) demo-v4-audit-pack
	$(MAKE_NO_PRINT) demo-v4-record

demo-v5-verify: shell-check
	$(PYTHON) scripts/generate_demo_v5_fixtures.py --out-dir "$(DEMO_V5_DIR)"
	for p in "$(DEMO_V5_HEALING_A)" "$(DEMO_V5_HEALING_B)" "$(DEMO_V5_BOUNDED_A)" "$(DEMO_V5_BOUNDED_B)" "$(DEMO_V5_GROWTH_A)" "$(DEMO_V5_GROWTH_B)" "$(DEMO_V5_TRANSITION_A)" "$(DEMO_V5_TRANSITION_B)"; do \
	  test -s "$$p"; \
	done
	python3 tests/test_demo_v5_evolution.py
	echo "== Demo V5 Verify: self_healing =="
	$(PYTHON) scripts/artifact_diff.py "$(DEMO_V5_HEALING_A)" "$(DEMO_V5_HEALING_B)"
	$(ARTIFACT_TOOL) inspect "$(DEMO_V5_HEALING_A)" --frames 4093:4100
	$(ARTIFACT_TOOL) inspect "$(DEMO_V5_HEALING_B)" --frames 4093:4100
	echo
	echo "== Demo V5 Verify: bounded_persistent =="
	$(PYTHON) scripts/artifact_diff.py "$(DEMO_V5_BOUNDED_A)" "$(DEMO_V5_BOUNDED_B)"
	$(ARTIFACT_TOOL) inspect "$(DEMO_V5_BOUNDED_A)" --frames 4093:4100
	$(ARTIFACT_TOOL) inspect "$(DEMO_V5_BOUNDED_B)" --frames 4093:4100
	echo
	echo "== Demo V5 Verify: monotonic_growth =="
	$(PYTHON) scripts/artifact_diff.py "$(DEMO_V5_GROWTH_A)" "$(DEMO_V5_GROWTH_B)"
	$(ARTIFACT_TOOL) inspect "$(DEMO_V5_GROWTH_A)" --frames 4093:4100
	$(ARTIFACT_TOOL) inspect "$(DEMO_V5_GROWTH_B)" --frames 4093:4100
	echo
	echo "== Demo V5 Verify: region_transition =="
	$(PYTHON) scripts/artifact_diff.py "$(DEMO_V5_TRANSITION_A)" "$(DEMO_V5_TRANSITION_B)"
	$(ARTIFACT_TOOL) inspect "$(DEMO_V5_TRANSITION_A)" --frames 4093:4101
	$(ARTIFACT_TOOL) inspect "$(DEMO_V5_TRANSITION_B)" --frames 4093:4101

demo-v5-audit-pack: shell-check
	$(PYTHON) scripts/generate_demo_v5_fixtures.py --out-dir "$(DEMO_V5_DIR)"
	echo "== Demo V5 Audit Pack =="
	echo "Git"
	if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then \
	  echo "commit=$$(git rev-parse HEAD)"; \
	else \
	  echo "commit=unavailable"; \
	fi
	echo
	echo "Fixtures"
	for p in "$(DEMO_V5_HEALING_A)" "$(DEMO_V5_HEALING_B)" "$(DEMO_V5_BOUNDED_A)" "$(DEMO_V5_BOUNDED_B)" "$(DEMO_V5_GROWTH_A)" "$(DEMO_V5_GROWTH_B)" "$(DEMO_V5_TRANSITION_A)" "$(DEMO_V5_TRANSITION_B)"; do \
	  test -s "$$p"; \
	  echo "$$p"; \
	  $(ARTIFACT_TOOL) hash "$$p"; \
	done
	echo
	echo "Self Healing Diff"
	$(PYTHON) scripts/artifact_diff.py "$(DEMO_V5_HEALING_A)" "$(DEMO_V5_HEALING_B)"
	echo
	echo "Bounded Persistent Diff"
	$(PYTHON) scripts/artifact_diff.py "$(DEMO_V5_BOUNDED_A)" "$(DEMO_V5_BOUNDED_B)"
	echo
	echo "Monotonic Growth Diff"
	$(PYTHON) scripts/artifact_diff.py "$(DEMO_V5_GROWTH_A)" "$(DEMO_V5_GROWTH_B)"
	echo
	echo "Region Transition Diff"
	$(PYTHON) scripts/artifact_diff.py "$(DEMO_V5_TRANSITION_A)" "$(DEMO_V5_TRANSITION_B)"

demo-v5-record: shell-check
	{ \
	  mkdir -p "$(DEMO_V5_RUNLOG_DIR)"; \
	  SHORT_SHA="$$(git rev-parse --short HEAD 2>/dev/null || echo unknown)"; \
	  FULL_SHA="$$(git rev-parse HEAD 2>/dev/null || echo unavailable)"; \
	  UTC_DATE="$$(date -u '+%Y-%m-%dT%H:%M:%SZ')"; \
	  OUT_FILE="$(DEMO_V5_RUNLOG_DIR)/demo_v5_release_$${SHORT_SHA}.txt"; \
	  { \
	    echo "== Demo V5 Verification Record =="; \
	    echo "commit=$${FULL_SHA}"; \
	    echo "date=$${UTC_DATE}"; \
	    echo; \
	    echo "== Verification =="; \
	    $(MAKE_NO_PRINT) demo-v5-verify; \
	    echo; \
	    echo "== Audit Pack =="; \
	    $(MAKE_NO_PRINT) demo-v5-audit-pack; \
	  } 2>&1 | tee "$${OUT_FILE}"; \
	  echo "Wrote $${OUT_FILE}"; \
	}

demo-v5-release: shell-check
	$(MAKE_NO_PRINT) demo-v5-verify
	$(MAKE_NO_PRINT) demo-v5-audit-pack
	$(MAKE_NO_PRINT) demo-v5-record

replay-demo-audit: shell-check
	$(MAKE_NO_PRINT) demo-v2-fixture-verify
	$(MAKE_NO_PRINT) demo-v3-verify
	$(MAKE_NO_PRINT) demo-v4-verify
	$(MAKE_NO_PRINT) demo-v5-verify
	$(MAKE_NO_PRINT) demo-captured-verify

debug-session:
	@echo "Killing old st-util..."
	-@killall -q st-util || pkill -x st-util || true
	@echo "Starting fresh st-util..."
	@st-util -u -F 200K -p 4242 > st-util.log 2>&1 & \
	sleep 1; \
	echo "Launching GDB..."; \
	gdb-multiarch -q -x scripts/tim2_debug.gdb

tim2-smoke:
	@echo "Killing old st-util..."
	-@killall -q st-util || pkill -x st-util || true
	$(MAKE) flash-ur
	$(MAKE) flash-compare-ur
	$(MAKE) debug-session

check-workspace:
	$(CARGO) fmt -- --check
	$(CARGO) clippy --workspace --locked -- -D warnings
	$(CARGO) check --workspace --locked

$(AUDIT_BIN):
	$(CARGO) build --locked -p $(DPW4_PKG) --features $(CLI_FEATURE) --bin substrate_probe

doc-link-check:
	$(XTASK_WORKFLOW) doc-link-check

test:
	$(CARGO) test --workspace --locked
	$(MAKE_NO_PRINT) authoritative-replay-cli-tests

authoritative-replay-cli-tests:
	$(CARGO) test -p $(DPW4_PKG) --features $(CLI_FEATURE) --test precision_authoritative_surface --locked

parser-tests:
	$(XTASK_WORKFLOW) parser-tests

replay-tool-tests:
	$(XTASK_WORKFLOW) replay-tool-tests

replay-tests:
	$(XTASK_WORKFLOW) replay-tests

demo-divergence:
	@test -x "$(SHELL)"
	@TMP_DIR="/tmp/precision_signal_demo_divergence"; \
	BASELINE_A="$$TMP_DIR/quant_probe_baseline_run1.rpl"; \
	BASELINE_B="$$TMP_DIR/quant_probe_baseline_run2.rpl"; \
	QUANTIZED_A="$$TMP_DIR/quant_probe_quantized_run1.rpl"; \
	mkdir -p "$$TMP_DIR"; \
	PYTHONPATH=. python3 -m experiments.quantization_probe.generate_probe_artifact --mode baseline --out "$$BASELINE_A" >/dev/null; \
	PYTHONPATH=. python3 -m experiments.quantization_probe.generate_probe_artifact --mode baseline --out "$$BASELINE_B" >/dev/null; \
	PYTHONPATH=. python3 -m experiments.quantization_probe.generate_probe_artifact --mode quantized --out "$$QUANTIZED_A" >/dev/null; \
	cmp -s "$$BASELINE_A" "$$BASELINE_B"; \
	DIFF_OUT="$$(PYTHONPATH=. $(PYTHON) scripts/artifact_diff.py "$$BASELINE_A" "$$QUANTIZED_A")"; \
	FIRST_DIVERGENCE_FRAME="$$(printf '%s\n' "$$DIFF_OUT" | awk -F': ' '/^first_divergence_frame:/ {print $$2; exit}')"; \
	CLASSIFICATION="$$(printf '%s\n' "$$DIFF_OUT" | awk -F': ' '/^shape_class:/ {print $$2; exit}')"; \
	test -n "$$FIRST_DIVERGENCE_FRAME"; \
	test -n "$$CLASSIFICATION"; \
	echo "first_divergence_frame = $$FIRST_DIVERGENCE_FRAME"; \
	echo "classification = $$CLASSIFICATION"; \
	echo "baseline_invariant = true"

gate:
	$(CARGO) run --locked --release -p $(DPW4_PKG) --features $(CLI_FEATURE) --bin $(SIG_UTIL_BIN) -- validate --mode quick

gate-full:
	$(CARGO) run --locked --release -p $(DPW4_PKG) --features $(CLI_FEATURE) --bin $(SIG_UTIL_BIN) -- validate --mode full

ci-local:
	$(XTASK_WORKFLOW) ci-local

conformance-audit: $(AUDIT_BIN)
	echo "Audit: Verifying current substrate twin invariant"
	rm -f .audit_stdout
	mkdir -p artifacts
	$(AUDIT_BIN) --target "$(AUDIT_TARGET)" > .audit_stdout
	test "$$(wc -c < .audit_stdout)" -gt 0
	test "$$(sed -n '7p' .audit_stdout)" != ""
	RUN_ID="$$(sed -n '7p' .audit_stdout | sed 's#^ARTIFACT: artifacts/##')"
	test -n "$$RUN_ID"
	test -f "artifacts/$$RUN_ID/result.txt"
	cmp --silent .audit_stdout "artifacts/$$RUN_ID/result.txt"
	grep -Fx "TARGET: $(AUDIT_TARGET)" .audit_stdout >/dev/null
	echo "  [OK] stdout/result.txt twin"
	echo "  [OK] target string preserved bit-for-bit"
	$(MAKE_NO_PRINT) stream-purity

kill-switch-audit: $(AUDIT_BIN)
	echo "Audit: Verifying current silence-on-failure invariant"
	rm -f .audit_stdout
	rm -rf "artifacts/.tmp_$(AUDIT_FIXED_RUN_ID)" "artifacts/$(AUDIT_FIXED_RUN_ID)"
	mkdir -p "artifacts/.tmp_$(AUDIT_FIXED_RUN_ID)"
	set +e
	$(AUDIT_BIN) --force-id "$(AUDIT_FIXED_RUN_ID)" > .audit_stdout
	RET=$$?
	set -e
	test "$$RET" -eq 2
	[ ! -s .audit_stdout ]
	echo "  [OK] exit code 2 on collision"
	echo "  [OK] stdout suppressed on failure"
	rm -rf "artifacts/.tmp_$(AUDIT_FIXED_RUN_ID)"

stream-purity:
	echo "Audit: Verifying LF-only 7-line substrate stream"
	test -f .audit_stdout
	! od -An -t x1 .audit_stdout | grep -qi '0d'
	test "$$(od -An -t x1 .audit_stdout | tr ' ' '\n' | grep -c '^0a$$')" -eq 7
	echo "  [OK] no CR bytes"
	echo "  [OK] exactly 7 LF bytes"

clean:
	$(CARGO) clean -p $(FW_PKG) -p $(DPW4_PKG)

kani-gate:
	bash scripts/verify_kani.sh

kani-gate-tier2:
	RUN_TIER2=1 bash scripts/verify_kani.sh

kani-gate-tier3:
	RUN_TIER3=1 bash scripts/verify_kani.sh

verify-repro:
	bash scripts/verify_release_repro.sh
