# Deterministic shell path for this repo. If your host uses a different bash
# location (for example /bin/bash), invoke make with SHELL=/path/to/bash.
SHELL := /usr/bin/bash
.ONESHELL:
.SHELLFLAGS := --noprofile --norc -euo pipefail -c

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
FW_CAPTURE_DIR ?= artifacts/fw_capture_runs
FW_REPEAT_DIR ?= artifacts/fw_repeat_runs
REPLAY_SIGNAL_MODEL ?= phase8
REPLAY_BASELINE ?= artifacts/baseline.bin
REPLAY_RUN ?= artifacts/run.bin
REPLAY_REPEAT_RUNS ?= 5
REPLAY_REPEAT_DIR ?= artifacts/replay_runs
AUDIT_TARGET ?= substrate://probe/default
AUDIT_BIN := ./target/debug/substrate_probe
AUDIT_FIXED_RUN_ID := AUDIT_COLLISION
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

.PHONY: help help-all help-demos help-firmware fixture-drift-check shell-check stflash-check fw fw-bin flash flash-verify flash-compare flash-ur flash-verify-ur flash-compare-ur demo-signal demo-signal-flash demo-signal-host-baseline demo-signal-host-perturb demo-signal-pi-baseline demo-signal-pi-perturb demo-signal-diff fw-capture-check fw-repeat-check rpl0-replay-check rpl0-replay-repeat-check rpl0-replay-repeat-auto fw-gate firmware-release-check fw-release-archive release-1.7.0 release-bundle release-bundle-check capture-demo-A capture-demo-B demo-captured-verify demo-captured-release demo-divergence demo-v2-capture demo-v2-fixture-verify demo-v2-verify demo-v2-audit-pack demo-v2-record demo-v3-verify demo-v3-audit-pack demo-v3-record demo-v3-release demo-v4-verify demo-v4-audit-pack demo-v4-record demo-v4-release demo-v5-verify demo-v5-audit-pack demo-v5-record demo-v5-release demo-evidence-package replay-demo-audit debug-session tim2-smoke doc-link-check check-workspace test authoritative-replay-cli-tests parser-tests replay-tool-tests replay-tests gate gate-full ci-local conformance-audit kill-switch-audit stream-purity clean

help:
	echo "Active operator / release path:"
	echo "  make gate"
	echo "  make release-1.7.0"
	echo "  make release-bundle VERSION=1.7.0"
	echo "  make release-bundle-check VERSION=1.7.0"
	echo "  make doc-link-check"
	echo "  make check-workspace"
	echo "  make test"
	echo "  make help-all"

help-all:
	echo "Active operator / release:"
	echo "  make gate"
	echo "  make release-1.7.0"
	echo "  make release-bundle VERSION=1.7.0"
	echo "  make release-bundle-check VERSION=1.7.0"
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
	$(MAKE) --no-print-directory help-demos
	echo
	$(MAKE) --no-print-directory help-firmware

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
	cargo run --quiet -p xtask -- workflow fixture-drift-check

demo-evidence-package:
	cargo run --quiet -p xtask -- workflow demo-evidence-package

release-1.7.0:
	@REL_DIR="docs/verification/releases/1.7.0"; \
	test -f "$$REL_DIR/kani_evidence.txt" && test -s "$$REL_DIR/kani_evidence.txt" || { \
	  echo "[release-1.7.0] FAIL missing or empty $$REL_DIR/kani_evidence.txt"; \
	  exit 1; \
	}
	@REL_DIR="docs/verification/releases/1.7.0"; \
	echo "--- [GATE 1/4] Functional Validation ---" && \
	$(MAKE) --no-print-directory gate > "$$REL_DIR/make_gate.txt" 2>&1 && \
	echo "--- [GATE 2/4] Evidence Packaging ---" && \
	$(MAKE) --no-print-directory demo-evidence-package > "$$REL_DIR/make_demo_evidence_package.txt" 2>&1 && \
	echo "--- [GATE 3/4] Documentation Integrity ---" && \
	$(MAKE) --no-print-directory doc-link-check > "$$REL_DIR/make_doc_link_check.txt" 2>&1 && \
	echo "--- [GATE 4/4] Reproducibility Record ---" && \
	RELEASE_EVIDENCE_DIR="$$REL_DIR" bash scripts/verify_release_repro.sh > "$$REL_DIR/release_reproducibility.txt" 2>&1 && \
	echo "--- [AUDIT] Bundle Coherence Check ---" && \
	$(MAKE) --no-print-directory release-bundle-check VERSION=1.7.0 > "$$REL_DIR/make_release_bundle_check.txt" 2>&1

shell-check:
	test -x "$(SHELL)"

stflash-check:
	ST="$$(command -v $(STFLASH))"
	test -n "$$ST"
	test -x "$$ST"
	echo "STFLASH=$$ST"

fw: shell-check
	cargo build -p $(FW_PKG) --target $(FW_TARGET) --locked $(FW_FEATURES_ARG)
	test -f "$(FW_ELF)"
	SECTIONS="$$(readelf -S "$(FW_ELF)")"
	echo "$$SECTIONS" | grep -q '\.text'
	echo "$$SECTIONS" | grep -Eq '\.vector_table|\.isr_vector'
	EP="$$(readelf -h "$(FW_ELF)" | awk '/Entry point address:/ {print $$NF; exit}')"
	test "$$EP" != "0x0"

fw-bin: fw
	rm -f "$(FW_BIN)"
	cargo objcopy -p $(FW_PKG) --target $(FW_TARGET) $(FW_FEATURES_ARG) -- -O binary "$(FW_BIN)"
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
	python3 - "$(FLASH_HEAD)" <<'PY'
	import struct, sys
	p = sys.argv[1]
	msp, rv = struct.unpack("<II", open(p, "rb").read(8))
	assert 0x20000000 <= msp <= 0x20020000, hex(msp)
	assert (rv & 1) == 1, hex(rv)
	rv &= ~1
	assert 0x08000000 <= rv <= 0x08080000, hex(rv)
	print("OK: MSP", hex(msp), "ResetVec", hex(rv | 1))
	PY

flash-verify-ur: fw-bin stflash-check
	rm -f "$(FLASH_HEAD)"
	$(STFLASH) --connect-under-reset --freq=200K read "$(FLASH_HEAD)" "$(FLASH_ADDR)" 64
	test -s "$(FLASH_HEAD)"
	hexdump -C "$(FLASH_HEAD)" | sed -n '1,4p'
	python3 - "$(FLASH_HEAD)" <<'PY'
	import struct, sys
	p = sys.argv[1]
	msp, rv = struct.unpack("<II", open(p, "rb").read(8))
	assert 0x20000000 <= msp <= 0x20020000, hex(msp)
	assert (rv & 1) == 1, hex(rv)
	rv &= ~1
	assert 0x08000000 <= rv <= 0x08080000, hex(rv)
	print("OK: MSP", hex(msp), "ResetVec", hex(rv | 1))
	PY

flash-compare: fw-bin stflash-check
	SIZE="$$(wc -c < "$(FW_BIN)")"
	test "$$SIZE" -gt 0
	rm -f "$(FLASH_FULL)"
	$(STFLASH) read "$(FLASH_FULL)" "$(FLASH_ADDR)" "$$SIZE"
	test -s "$(FLASH_FULL)"
	cmp -s "$(FW_BIN)" "$(FLASH_FULL)" || { echo "flash-compare FAIL: device != $(FW_BIN)"; false; }

flash-compare-ur: fw-bin stflash-check
	SIZE="$$(wc -c < "$(FW_BIN)")"
	test "$$SIZE" -gt 0
	rm -f "$(FLASH_FULL)"
	$(STFLASH) --connect-under-reset --freq=200K read "$(FLASH_FULL)" "$(FLASH_ADDR)" "$$SIZE"
	test -s "$(FLASH_FULL)"
	cmp -s "$(FW_BIN)" "$(FLASH_FULL)" || { echo "flash-compare-ur FAIL: device != $(FW_BIN)"; false; }

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
	python3 scripts/csv_capture.py --serial "$(SERIAL)" --out "$(SIGNAL_BASELINE_CSV)"

demo-signal-host-perturb:
	rm -f "$(SIGNAL_OBSERVED_CSV)"
	echo "Starting host capture for perturbation on $(SERIAL)."
	echo "Next steps:"
	echo "  1. Listener is starting now."
	echo "  2. Reset STM32 after the listener is attached."
	echo "  3. Run on the Pi: make demo-signal-pi-perturb"
	python3 scripts/csv_capture.py --serial "$(SERIAL)" --out "$(SIGNAL_OBSERVED_CSV)"

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
	python3 scripts/pi_emitter.py --mode baseline --frames "$(SIGNAL_FRAMES)" --perturb-frame "$(SIGNAL_PERTURB_FRAME)" $(if $(SIGNAL_REPEAT_SECONDS),--repeat-seconds "$(SIGNAL_REPEAT_SECONDS)")

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
	python3 scripts/pi_emitter.py --mode perturb --frames "$(SIGNAL_FRAMES)" --perturb-frame "$(SIGNAL_PERTURB_FRAME)" $(if $(SIGNAL_REPEAT_SECONDS),--repeat-seconds "$(SIGNAL_REPEAT_SECONDS)")

demo-signal-diff:
	python3 scripts/interval_diff.py "$(SIGNAL_BASELINE_CSV)" "$(SIGNAL_OBSERVED_CSV)"

# Active firmware hardware path:
# capture waits for STATE,CAPTURE_DONE,138 and validates index,interval_us CSV via replay-host.
fw-capture-check:
	test -n "$(SERIAL)" || { echo "SERIAL not set"; exit 1; }
	rm -rf "$(FW_CAPTURE_DIR)"
	SERIAL="$(SERIAL)" python3 scripts/replay_repeat_check.py \
	  --runs 1 \
	  --reset-mode "$(FW_GATE_RESET_MODE)" \
	  --timeout "$(FW_CAPTURE_TIMEOUT)" \
	  --artifacts-dir "$(FW_CAPTURE_DIR)"

fw-repeat-check:
	test -n "$(SERIAL)" || { echo "SERIAL not set"; exit 1; }
	rm -rf "$(FW_REPEAT_DIR)"
	SERIAL="$(SERIAL)" python3 scripts/replay_repeat_check.py \
	  --runs "$(REPLAY_REPEAT_RUNS)" \
	  --reset-mode "$(FW_GATE_RESET_MODE)" \
	  --timeout "$(FW_CAPTURE_TIMEOUT)" \
	  --artifacts-dir "$(FW_REPEAT_DIR)"

# Retained historical RPL0 operator path:
# capture waits for replay header; operator presses reset once after listener starts.
rpl0-replay-check:
	export PYTHONPATH="$(CURDIR)$${PYTHONPATH:+:$${PYTHONPATH}}"
	$(MAKE) flash-ur
	SERIAL="$(SERIAL)" python3 scripts/artifact_tool.py capture --quick --out "$(REPLAY_RUN)"
	python3 scripts/artifact_tool.py verify "$(REPLAY_RUN)" --signal-model "$(REPLAY_SIGNAL_MODEL)"
	python3 scripts/artifact_tool.py compare "$(REPLAY_BASELINE)" "$(REPLAY_RUN)"

rpl0-replay-repeat-check:
	export PYTHONPATH="$(CURDIR)$${PYTHONPATH:+:$${PYTHONPATH}}"
	SERIAL="$(SERIAL)" python3 scripts/repeat_capture.py \
	  --contract rpl0 \
	  --runs "$(REPLAY_REPEAT_RUNS)" \
	  --signal-model "$(REPLAY_SIGNAL_MODEL)" \
	  --artifacts-dir "$(REPLAY_REPEAT_DIR)"

rpl0-replay-repeat-auto: stflash-check
	export PYTHONPATH="$(CURDIR)$${PYTHONPATH:+:$${PYTHONPATH}}"
	SERIAL="$(SERIAL)" python3 scripts/repeat_capture.py \
	  --contract rpl0 \
	  --runs "$(REPLAY_REPEAT_RUNS)" \
	  --signal-model "$(REPLAY_SIGNAL_MODEL)" \
	  --reset-mode stlink \
	  --stflash "$(STFLASH)" \
	  --artifacts-dir "$(REPLAY_REPEAT_DIR)"

fw-gate:
	@test -n "$(SERIAL)" || { echo "SERIAL is required"; exit 1; }
	@test "$(FW_GATE_RESET_MODE)" = "manual" || { echo "FW_GATE_RESET_MODE must be manual for firmware gate"; exit 1; }
	$(MAKE) check-workspace
	$(MAKE) test
	bash scripts/verify_kani.sh
	$(MAKE) gate
	$(MAKE) fw
	$(MAKE) fw-bin
	$(MAKE) flash-ur SERIAL="$(SERIAL)"
	$(MAKE) flash-verify-ur
	$(MAKE) flash-compare-ur
	$(MAKE) fw-capture-check SERIAL="$(SERIAL)"
	$(MAKE) fw-repeat-check SERIAL="$(SERIAL)" REPLAY_REPEAT_RUNS=3

firmware-release-check: fw-gate
	@CAPTURE_DIR="$$(ls -dt "$(FW_CAPTURE_DIR)"/run_* | head -n1)"; \
	REPEAT_DIR="$$(ls -dt "$(FW_REPEAT_DIR)"/run_* | head -n1)"; \
	echo "CAPTURE_DIR=$$CAPTURE_DIR"; \
	echo "REPEAT_DIR=$$REPEAT_DIR"; \
	echo; \
	echo "== capture files =="; \
	ls -lah "$$CAPTURE_DIR"; \
	echo; \
	echo "== capture manifest =="; \
	cat "$$CAPTURE_DIR/interval_capture_manifest_v1.txt"; \
	echo; \
	echo "== repeat files =="; \
	ls -lah "$$REPEAT_DIR"; \
	echo; \
	echo "== repeat csv sha256 summary =="; \
	cat "$$REPEAT_DIR/csv_sha256_summary.txt"; \
	echo; \
	echo "== repeat imported sha256 summary =="; \
	cat "$$REPEAT_DIR/imported_artifact_sha256_summary.txt"; \
	echo; \
	echo "== repeat manifest =="; \
	cat "$$REPEAT_DIR/interval_capture_manifest_v1.txt"

fw-release-archive:
	@test -n "$(SERIAL)" || { echo "SERIAL is required"; exit 1; }
	@test -n "$(VERSION)" || { echo "VERSION is required"; exit 1; }
	$(MAKE) firmware-release-check SERIAL="$(SERIAL)"
	@CAPTURE_DIR="$$(ls -dt "$(FW_CAPTURE_DIR)"/run_* | head -n1)"; \
	REPEAT_DIR="$$(ls -dt "$(FW_REPEAT_DIR)"/run_* | head -n1)"; \
	REL_DIR="docs/verification/releases/$(VERSION)"; \
	mkdir -p "$$REL_DIR"; \
	rm -rf "$$REL_DIR/fw_capture" "$$REL_DIR/fw_repeat"; \
	cp -R "$$CAPTURE_DIR" "$$REL_DIR/fw_capture"; \
	cp -R "$$REPEAT_DIR" "$$REL_DIR/fw_repeat"; \
	sha256sum "$$CAPTURE_DIR"/run_*.csv "$$CAPTURE_DIR"/run_*.imported.rpl > "$$REL_DIR/fw_capture_hash_check.txt"; \
	sha256sum "$$REPEAT_DIR"/run_*.csv "$$REPEAT_DIR"/run_*.imported.rpl > "$$REL_DIR/fw_repeat_hash_check.txt"; \
	echo "# Firmware Release Evidence ($(VERSION))" > "$$REL_DIR/firmware_release_evidence.md"; \
	echo "" >> "$$REL_DIR/firmware_release_evidence.md"; \
	echo "CAPTURE_DIR=$$CAPTURE_DIR" >> "$$REL_DIR/firmware_release_evidence.md"; \
	echo "REPEAT_DIR=$$REPEAT_DIR" >> "$$REL_DIR/firmware_release_evidence.md"; \
	echo "" >> "$$REL_DIR/firmware_release_evidence.md"; \
	echo "## capture manifest" >> "$$REL_DIR/firmware_release_evidence.md"; \
	cat "$$REL_DIR/fw_capture/interval_capture_manifest_v1.txt" >> "$$REL_DIR/firmware_release_evidence.md"; \
	echo "" >> "$$REL_DIR/firmware_release_evidence.md"; \
	echo "## repeat csv sha256 summary" >> "$$REL_DIR/firmware_release_evidence.md"; \
	cat "$$REL_DIR/fw_repeat/csv_sha256_summary.txt" >> "$$REL_DIR/firmware_release_evidence.md"; \
	echo "" >> "$$REL_DIR/firmware_release_evidence.md"; \
	echo "## repeat imported sha256 summary" >> "$$REL_DIR/firmware_release_evidence.md"; \
	cat "$$REL_DIR/fw_repeat/imported_artifact_sha256_summary.txt" >> "$$REL_DIR/firmware_release_evidence.md"; \
	echo "" >> "$$REL_DIR/firmware_release_evidence.md"; \
	echo "## repeat manifest" >> "$$REL_DIR/firmware_release_evidence.md"; \
	cat "$$REL_DIR/fw_repeat/interval_capture_manifest_v1.txt" >> "$$REL_DIR/firmware_release_evidence.md"; \
	echo "" >> "$$REL_DIR/firmware_release_evidence.md"; \
	echo "## capture hash check" >> "$$REL_DIR/firmware_release_evidence.md"; \
	cat "$$REL_DIR/fw_capture_hash_check.txt" >> "$$REL_DIR/firmware_release_evidence.md"; \
	echo "" >> "$$REL_DIR/firmware_release_evidence.md"; \
	echo "## repeat hash check" >> "$$REL_DIR/firmware_release_evidence.md"; \
	cat "$$REL_DIR/fw_repeat_hash_check.txt" >> "$$REL_DIR/firmware_release_evidence.md"
	python3 scripts/check_release_bundle.py --version "$(VERSION)"

release-bundle:
	@test -n "$(VERSION)" || { echo "VERSION is required"; exit 1; }
	@REL_DIR="docs/verification/releases/$(VERSION)"; \
	WS_VERSION="$$(awk -F'"' '/^version = "/ {print $$2; exit}' Cargo.toml)"; \
	if [[ "$$WS_VERSION" != "$(VERSION)" ]]; then \
	  echo "[release-bundle] FAIL metadata: workspace version=$$WS_VERSION, requested VERSION=$(VERSION)"; \
	  exit 1; \
	fi; \
	mkdir -p "$$REL_DIR"; \
	echo "[release-bundle] 1/6 cargo_check_dpw4_thumb_locked.txt"; \
	if ! cargo check --locked -p dpw4 --target thumbv7em-none-eabihf > "$$REL_DIR/cargo_check_dpw4_thumb_locked.txt" 2>&1; then \
	  echo "[release-bundle] FAIL phase=1 file=$$REL_DIR/cargo_check_dpw4_thumb_locked.txt"; \
	  exit 1; \
	fi; \
	echo "[release-bundle] 2/6 kani_evidence.txt"; \
	if ! NO_COLOR=1 bash scripts/verify_kani.sh > "$$REL_DIR/kani_evidence.txt" 2>&1; then \
	  echo "[release-bundle] FAIL phase=2 file=$$REL_DIR/kani_evidence.txt"; \
	  exit 1; \
	fi; \
	echo "[release-bundle] 3/6 make_gate.txt"; \
	if ! $(MAKE) --no-print-directory gate > "$$REL_DIR/make_gate.txt" 2>&1; then \
	  echo "[release-bundle] FAIL phase=3 file=$$REL_DIR/make_gate.txt"; \
	  exit 1; \
	fi; \
	echo "[release-bundle] 4/6 release_reproducibility.txt"; \
	if ! RELEASE_EVIDENCE_DIR="$$REL_DIR" bash scripts/verify_release_repro.sh > "$$REL_DIR/release_reproducibility.txt" 2>&1; then \
	  echo "[release-bundle] FAIL phase=4 file=$$REL_DIR/release_reproducibility.txt"; \
	  exit 1; \
	fi; \
	echo "[release-bundle] 5/6 stale prior-release guard"; \
	if ! python3 scripts/release_bundle_guard_stale.py --dir "$$REL_DIR" --version "$(VERSION)"; then \
	  echo "[release-bundle] FAIL phase=5 stale prior-release workspace evidence"; \
	  exit 1; \
	fi; \
	echo "[release-bundle] 6/6 control-byte guard (non-lossy)"; \
	if ! python3 scripts/release_bundle_guard_control_bytes.py --dir "$$REL_DIR"; then \
	  echo "[release-bundle] FAIL phase=6 control bytes detected"; \
	  exit 1; \
	fi; \
	echo "[release-bundle] OK bundle generated: $$REL_DIR"

release-bundle-check:
	@test -n "$(VERSION)" || { echo "VERSION is required"; exit 1; }
	python3 scripts/check_release_bundle.py --version "$(VERSION)"

capture-demo-A: shell-check stflash-check
	echo "== Captured Divergence Demo: Run A (canonical firmware) =="
	mkdir -p "$(DEMO_CAPTURED_A_DIR)"
	test -n "$(SERIAL)" || { echo "SERIAL not set"; exit 1; }
	$(MAKE) FW_FEATURES= flash-ur
	SERIAL="$(SERIAL)" python3 scripts/repeat_capture.py \
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
	SERIAL="$(SERIAL)" python3 scripts/repeat_capture.py \
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
	python3 scripts/artifact_diff.py "$(DEMO_CAPTURED_A)" "$(DEMO_CAPTURED_B)"

demo-captured-release: demo-captured-verify
	echo "Captured divergence evidence verified."

demo-v2-capture: shell-check stflash-check
	echo "== Demo V2 Capture: Canonical =="
	mkdir -p "$(DEMO_V2_CAPTURE_A_DIR)" "$(DEMO_V2_CAPTURE_B_DIR)"
	$(MAKE) FW_FEATURES= flash-ur
	SERIAL="$(SERIAL)" python3 scripts/repeat_capture.py \
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
	SERIAL="$(SERIAL)" python3 scripts/repeat_capture.py \
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
	python3 scripts/artifact_diff.py "$(DEMO_V2_FIXTURE_A)" "$(DEMO_V2_FIXTURE_B)"
	python3 scripts/artifact_tool.py inspect "$(DEMO_V2_FIXTURE_A)" --frames 4093:4100
	python3 scripts/artifact_tool.py inspect "$(DEMO_V2_FIXTURE_B)" --frames 4093:4100
	python3 scripts/artifact_tool.py verify "$(DEMO_V2_FIXTURE_A)" --signal-model "$(REPLAY_SIGNAL_MODEL)"
	if python3 scripts/artifact_tool.py verify "$(DEMO_V2_FIXTURE_B)" --signal-model "$(REPLAY_SIGNAL_MODEL)"; then \
	  echo "ERROR: expected $(REPLAY_SIGNAL_MODEL) verification failure for $(DEMO_V2_FIXTURE_B)"; \
	  exit 1; \
	else \
	  echo "Expected failure observed: $(DEMO_V2_FIXTURE_B) with signal-model $(REPLAY_SIGNAL_MODEL)"; \
	fi
	python3 scripts/artifact_tool.py verify "$(DEMO_V2_FIXTURE_B)" --signal-model none

demo-v2-verify: demo-v2-fixture-verify
	echo "== Demo V2 Verify: Captured =="
	if [[ ! -s "$(DEMO_V2_CAPTURED_A)" || ! -s "$(DEMO_V2_CAPTURED_B)" ]]; then \
	  echo "Captured artifacts missing; run 'make demo-v2-capture' to generate $(DEMO_V2_CAPTURED_A) and $(DEMO_V2_CAPTURED_B)"; \
	  exit 1; \
	fi
	python3 scripts/artifact_diff.py "$(DEMO_V2_CAPTURED_A)" "$(DEMO_V2_CAPTURED_B)"
	python3 scripts/artifact_tool.py inspect "$(DEMO_V2_CAPTURED_A)" --frames 4093:4100
	python3 scripts/artifact_tool.py inspect "$(DEMO_V2_CAPTURED_B)" --frames 4093:4100
	python3 scripts/artifact_tool.py verify "$(DEMO_V2_CAPTURED_A)" --signal-model "$(REPLAY_SIGNAL_MODEL)"
	if python3 scripts/artifact_tool.py verify "$(DEMO_V2_CAPTURED_B)" --signal-model "$(REPLAY_SIGNAL_MODEL)"; then \
	  echo "ERROR: expected $(REPLAY_SIGNAL_MODEL) verification failure for $(DEMO_V2_CAPTURED_B)"; \
	  exit 1; \
	else \
	  echo "Expected failure observed: $(DEMO_V2_CAPTURED_B) with signal-model $(REPLAY_SIGNAL_MODEL)"; \
	fi
	python3 scripts/artifact_tool.py verify "$(DEMO_V2_CAPTURED_B)" --signal-model none

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
	python3 scripts/artifact_tool.py hash "$(DEMO_V2_FIXTURE_A)"
	python3 scripts/artifact_tool.py hash "$(DEMO_V2_FIXTURE_B)"
	echo
	echo "Captured Artifacts"
	if [[ -s "$(DEMO_V2_CAPTURED_A)" && -s "$(DEMO_V2_CAPTURED_B)" ]]; then \
	  echo "$(DEMO_V2_CAPTURED_A)"; \
	  echo "$(DEMO_V2_CAPTURED_B)"; \
	  python3 scripts/artifact_tool.py hash "$(DEMO_V2_CAPTURED_A)"; \
	  python3 scripts/artifact_tool.py hash "$(DEMO_V2_CAPTURED_B)"; \
	  CAPTURED_PRESENT=1; \
	else \
	  echo "captured artifacts absent (run 'make demo-v2-capture')"; \
	  CAPTURED_PRESENT=0; \
	fi; \
	echo; \
	echo "Fixture Diff"; \
	python3 scripts/artifact_diff.py "$(DEMO_V2_FIXTURE_A)" "$(DEMO_V2_FIXTURE_B)"; \
	echo; \
	echo "Fixture Inspect A"; \
	python3 scripts/artifact_tool.py inspect "$(DEMO_V2_FIXTURE_A)" --frames 4093:4100; \
	echo; \
	echo "Fixture Inspect B"; \
	python3 scripts/artifact_tool.py inspect "$(DEMO_V2_FIXTURE_B)" --frames 4093:4100; \
	echo; \
	echo "Fixture Verify"; \
	python3 scripts/artifact_tool.py verify "$(DEMO_V2_FIXTURE_A)" --signal-model "$(REPLAY_SIGNAL_MODEL)"; \
	if python3 scripts/artifact_tool.py verify "$(DEMO_V2_FIXTURE_B)" --signal-model "$(REPLAY_SIGNAL_MODEL)"; then \
	  echo "ERROR: expected $(REPLAY_SIGNAL_MODEL) verification failure for $(DEMO_V2_FIXTURE_B)"; \
	  exit 1; \
	else \
	  echo "Expected failure observed: $(DEMO_V2_FIXTURE_B) with signal-model $(REPLAY_SIGNAL_MODEL)"; \
	fi; \
	python3 scripts/artifact_tool.py verify "$(DEMO_V2_FIXTURE_B)" --signal-model none; \
	if [[ "$$CAPTURED_PRESENT" -eq 1 ]]; then \
	  echo; \
	  echo "Captured Diff"; \
	  python3 scripts/artifact_diff.py "$(DEMO_V2_CAPTURED_A)" "$(DEMO_V2_CAPTURED_B)"; \
	  echo; \
	  echo "Captured Inspect A"; \
	  python3 scripts/artifact_tool.py inspect "$(DEMO_V2_CAPTURED_A)" --frames 4093:4100; \
	  echo; \
	  echo "Captured Inspect B"; \
	  python3 scripts/artifact_tool.py inspect "$(DEMO_V2_CAPTURED_B)" --frames 4093:4100; \
	  echo; \
	  echo "Captured Verify"; \
	  python3 scripts/artifact_tool.py verify "$(DEMO_V2_CAPTURED_A)" --signal-model "$(REPLAY_SIGNAL_MODEL)"; \
	  if python3 scripts/artifact_tool.py verify "$(DEMO_V2_CAPTURED_B)" --signal-model "$(REPLAY_SIGNAL_MODEL)"; then \
	    echo "ERROR: expected $(REPLAY_SIGNAL_MODEL) verification failure for $(DEMO_V2_CAPTURED_B)"; \
	    exit 1; \
	  else \
	    echo "Expected failure observed: $(DEMO_V2_CAPTURED_B) with signal-model $(REPLAY_SIGNAL_MODEL)"; \
	  fi; \
	  python3 scripts/artifact_tool.py verify "$(DEMO_V2_CAPTURED_B)" --signal-model none; \
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
	  $(MAKE) --no-print-directory demo-v2-verify
	  echo
	  echo "== Audit Pack =="
	  $(MAKE) --no-print-directory demo-v2-audit-pack
	} 2>&1 | tee "$${OUT_FILE}"
	echo "Wrote $${OUT_FILE}"

demo-v3-verify: shell-check
	python3 scripts/generate_demo_v3_fixtures.py --out-dir "$(DEMO_V3_DIR)"
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
	  out="$$(python3 scripts/artifact_diff.py "$$artifact_a" "$$artifact_b")"; \
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
	python3 scripts/artifact_tool.py inspect "$(DEMO_V3_TRANSIENT_A)" --frames 4093:4103
	python3 scripts/artifact_tool.py inspect "$(DEMO_V3_TRANSIENT_B)" --frames 4093:4103
	echo "== Demo V3 Verify: persistent_offset =="
	check_classification "$(DEMO_V3_OFFSET_A)" "$(DEMO_V3_OFFSET_B)" persistent_offset 4096
	python3 scripts/artifact_tool.py inspect "$(DEMO_V3_OFFSET_A)" --frames 4093:4103
	python3 scripts/artifact_tool.py inspect "$(DEMO_V3_OFFSET_B)" --frames 4093:4103
	echo "== Demo V3 Verify: rate_divergence =="
	check_classification "$(DEMO_V3_RATE_A)" "$(DEMO_V3_RATE_B)" rate_divergence 4096
	python3 scripts/artifact_tool.py inspect "$(DEMO_V3_RATE_A)" --frames 4093:4103
	python3 scripts/artifact_tool.py inspect "$(DEMO_V3_RATE_B)" --frames 4093:4103

demo-v3-audit-pack: shell-check
	python3 scripts/generate_demo_v3_fixtures.py --out-dir "$(DEMO_V3_DIR)"
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
	  python3 scripts/artifact_tool.py hash "$$p"; \
	done
	echo
	echo "Transient Diff"
	python3 scripts/artifact_diff.py "$(DEMO_V3_TRANSIENT_A)" "$(DEMO_V3_TRANSIENT_B)"
	echo
	echo "Transient Inspect A"
	python3 scripts/artifact_tool.py inspect "$(DEMO_V3_TRANSIENT_A)" --frames 4093:4103
	echo
	echo "Transient Inspect B"
	python3 scripts/artifact_tool.py inspect "$(DEMO_V3_TRANSIENT_B)" --frames 4093:4103
	echo
	echo "Persistent Offset Diff"
	python3 scripts/artifact_diff.py "$(DEMO_V3_OFFSET_A)" "$(DEMO_V3_OFFSET_B)"
	echo
	echo "Persistent Offset Inspect A"
	python3 scripts/artifact_tool.py inspect "$(DEMO_V3_OFFSET_A)" --frames 4093:4103
	echo
	echo "Persistent Offset Inspect B"
	python3 scripts/artifact_tool.py inspect "$(DEMO_V3_OFFSET_B)" --frames 4093:4103
	echo
	echo "Rate Divergence Diff"
	python3 scripts/artifact_diff.py "$(DEMO_V3_RATE_A)" "$(DEMO_V3_RATE_B)"
	echo
	echo "Rate Divergence Inspect A"
	python3 scripts/artifact_tool.py inspect "$(DEMO_V3_RATE_A)" --frames 4093:4103
	echo
	echo "Rate Divergence Inspect B"
	python3 scripts/artifact_tool.py inspect "$(DEMO_V3_RATE_B)" --frames 4093:4103

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
	  $(MAKE) --no-print-directory demo-v3-verify
	  echo
	  echo "== Audit Pack =="
	  $(MAKE) --no-print-directory demo-v3-audit-pack
	} 2>&1 | tee "$${OUT_FILE}"
	echo "Wrote $${OUT_FILE}"

demo-v3-release: shell-check
	$(MAKE) --no-print-directory demo-v3-verify
	$(MAKE) --no-print-directory demo-v3-audit-pack
	$(MAKE) --no-print-directory demo-v3-record

demo-v4-verify: shell-check
	python3 scripts/generate_demo_v4_fixtures.py --out-dir "$(DEMO_V4_DIR)"
	for p in "$(DEMO_V4_HEADER_A)" "$(DEMO_V4_HEADER_B)" "$(DEMO_V4_HEADER_SAMPLE_A)" "$(DEMO_V4_HEADER_SAMPLE_B)" "$(DEMO_V4_TIMER_A)" "$(DEMO_V4_TIMER_B)" "$(DEMO_V4_IRQ_A)" "$(DEMO_V4_IRQ_B)" "$(DEMO_V4_SAMPLE_A)" "$(DEMO_V4_SAMPLE_B)" "$(DEMO_V4_MIXED_A)" "$(DEMO_V4_MIXED_B)"; do \
	  test -s "$$p"; \
	done
	python3 tests/test_demo_v4_region_attribution.py
	echo "== Demo V4 Verify: header_schema =="
	python3 scripts/artifact_diff.py "$(DEMO_V4_HEADER_A)" "$(DEMO_V4_HEADER_B)"
	echo
	echo "== Demo V4 Verify: header_schema + sample_payload @ frame 0 =="
	python3 scripts/artifact_diff.py "$(DEMO_V4_HEADER_SAMPLE_A)" "$(DEMO_V4_HEADER_SAMPLE_B)"
	echo
	echo "== Demo V4 Verify: timer_delta =="
	python3 scripts/artifact_diff.py "$(DEMO_V4_TIMER_A)" "$(DEMO_V4_TIMER_B)"
	python3 scripts/artifact_tool.py inspect "$(DEMO_V4_TIMER_A)" --frames 4093:4103
	python3 scripts/artifact_tool.py inspect "$(DEMO_V4_TIMER_B)" --frames 4093:4103
	echo
	echo "== Demo V4 Verify: irq_state =="
	python3 scripts/artifact_diff.py "$(DEMO_V4_IRQ_A)" "$(DEMO_V4_IRQ_B)"
	python3 scripts/artifact_tool.py inspect "$(DEMO_V4_IRQ_A)" --frames 4093:4103
	python3 scripts/artifact_tool.py inspect "$(DEMO_V4_IRQ_B)" --frames 4093:4103
	echo
	echo "== Demo V4 Verify: sample_payload =="
	python3 scripts/artifact_diff.py "$(DEMO_V4_SAMPLE_A)" "$(DEMO_V4_SAMPLE_B)"
	python3 scripts/artifact_tool.py inspect "$(DEMO_V4_SAMPLE_A)" --frames 4093:4103
	python3 scripts/artifact_tool.py inspect "$(DEMO_V4_SAMPLE_B)" --frames 4093:4103
	echo
	echo "== Demo V4 Verify: mixed =="
	python3 scripts/artifact_diff.py "$(DEMO_V4_MIXED_A)" "$(DEMO_V4_MIXED_B)"
	python3 scripts/artifact_tool.py inspect "$(DEMO_V4_MIXED_A)" --frames 4093:4103
	python3 scripts/artifact_tool.py inspect "$(DEMO_V4_MIXED_B)" --frames 4093:4103

demo-v4-audit-pack: shell-check
	python3 scripts/generate_demo_v4_fixtures.py --out-dir "$(DEMO_V4_DIR)"
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
	  python3 scripts/artifact_tool.py hash "$$p"; \
	done
	echo
	echo "Header/Schema Diff"
	python3 scripts/artifact_diff.py "$(DEMO_V4_HEADER_A)" "$(DEMO_V4_HEADER_B)"
	echo
	echo "Header/Schema + Sample Payload @ Frame 0 Diff"
	python3 scripts/artifact_diff.py "$(DEMO_V4_HEADER_SAMPLE_A)" "$(DEMO_V4_HEADER_SAMPLE_B)"
	echo
	echo "Timer Delta Diff"
	python3 scripts/artifact_diff.py "$(DEMO_V4_TIMER_A)" "$(DEMO_V4_TIMER_B)"
	echo
	echo "IRQ State Diff"
	python3 scripts/artifact_diff.py "$(DEMO_V4_IRQ_A)" "$(DEMO_V4_IRQ_B)"
	echo
	echo "Sample Payload Diff"
	python3 scripts/artifact_diff.py "$(DEMO_V4_SAMPLE_A)" "$(DEMO_V4_SAMPLE_B)"
	echo
	echo "Mixed Diff"
	python3 scripts/artifact_diff.py "$(DEMO_V4_MIXED_A)" "$(DEMO_V4_MIXED_B)"

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
	    $(MAKE) --no-print-directory demo-v4-verify; \
	    echo; \
	    echo "== Audit Pack =="; \
	    $(MAKE) --no-print-directory demo-v4-audit-pack; \
	  } 2>&1 | tee "$${OUT_FILE}"; \
	  echo "Wrote $${OUT_FILE}"; \
	}

demo-v4-release: shell-check
	$(MAKE) --no-print-directory demo-v4-verify
	$(MAKE) --no-print-directory demo-v4-audit-pack
	$(MAKE) --no-print-directory demo-v4-record

demo-v5-verify: shell-check
	python3 scripts/generate_demo_v5_fixtures.py --out-dir "$(DEMO_V5_DIR)"
	for p in "$(DEMO_V5_HEALING_A)" "$(DEMO_V5_HEALING_B)" "$(DEMO_V5_BOUNDED_A)" "$(DEMO_V5_BOUNDED_B)" "$(DEMO_V5_GROWTH_A)" "$(DEMO_V5_GROWTH_B)" "$(DEMO_V5_TRANSITION_A)" "$(DEMO_V5_TRANSITION_B)"; do \
	  test -s "$$p"; \
	done
	python3 tests/test_demo_v5_evolution.py
	echo "== Demo V5 Verify: self_healing =="
	python3 scripts/artifact_diff.py "$(DEMO_V5_HEALING_A)" "$(DEMO_V5_HEALING_B)"
	python3 scripts/artifact_tool.py inspect "$(DEMO_V5_HEALING_A)" --frames 4093:4100
	python3 scripts/artifact_tool.py inspect "$(DEMO_V5_HEALING_B)" --frames 4093:4100
	echo
	echo "== Demo V5 Verify: bounded_persistent =="
	python3 scripts/artifact_diff.py "$(DEMO_V5_BOUNDED_A)" "$(DEMO_V5_BOUNDED_B)"
	python3 scripts/artifact_tool.py inspect "$(DEMO_V5_BOUNDED_A)" --frames 4093:4100
	python3 scripts/artifact_tool.py inspect "$(DEMO_V5_BOUNDED_B)" --frames 4093:4100
	echo
	echo "== Demo V5 Verify: monotonic_growth =="
	python3 scripts/artifact_diff.py "$(DEMO_V5_GROWTH_A)" "$(DEMO_V5_GROWTH_B)"
	python3 scripts/artifact_tool.py inspect "$(DEMO_V5_GROWTH_A)" --frames 4093:4100
	python3 scripts/artifact_tool.py inspect "$(DEMO_V5_GROWTH_B)" --frames 4093:4100
	echo
	echo "== Demo V5 Verify: region_transition =="
	python3 scripts/artifact_diff.py "$(DEMO_V5_TRANSITION_A)" "$(DEMO_V5_TRANSITION_B)"
	python3 scripts/artifact_tool.py inspect "$(DEMO_V5_TRANSITION_A)" --frames 4093:4101
	python3 scripts/artifact_tool.py inspect "$(DEMO_V5_TRANSITION_B)" --frames 4093:4101

demo-v5-audit-pack: shell-check
	python3 scripts/generate_demo_v5_fixtures.py --out-dir "$(DEMO_V5_DIR)"
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
	  python3 scripts/artifact_tool.py hash "$$p"; \
	done
	echo
	echo "Self Healing Diff"
	python3 scripts/artifact_diff.py "$(DEMO_V5_HEALING_A)" "$(DEMO_V5_HEALING_B)"
	echo
	echo "Bounded Persistent Diff"
	python3 scripts/artifact_diff.py "$(DEMO_V5_BOUNDED_A)" "$(DEMO_V5_BOUNDED_B)"
	echo
	echo "Monotonic Growth Diff"
	python3 scripts/artifact_diff.py "$(DEMO_V5_GROWTH_A)" "$(DEMO_V5_GROWTH_B)"
	echo
	echo "Region Transition Diff"
	python3 scripts/artifact_diff.py "$(DEMO_V5_TRANSITION_A)" "$(DEMO_V5_TRANSITION_B)"

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
	    $(MAKE) --no-print-directory demo-v5-verify; \
	    echo; \
	    echo "== Audit Pack =="; \
	    $(MAKE) --no-print-directory demo-v5-audit-pack; \
	  } 2>&1 | tee "$${OUT_FILE}"; \
	  echo "Wrote $${OUT_FILE}"; \
	}

demo-v5-release: shell-check
	$(MAKE) --no-print-directory demo-v5-verify
	$(MAKE) --no-print-directory demo-v5-audit-pack
	$(MAKE) --no-print-directory demo-v5-record

replay-demo-audit: shell-check
	$(MAKE) --no-print-directory demo-v2-fixture-verify
	$(MAKE) --no-print-directory demo-v3-verify
	$(MAKE) --no-print-directory demo-v4-verify
	$(MAKE) --no-print-directory demo-v5-verify
	$(MAKE) --no-print-directory demo-captured-verify

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
	cargo fmt -- --check
	cargo clippy --workspace --locked -- -D warnings
	cargo check --workspace --locked

$(AUDIT_BIN):
	cargo build --locked -p dpw4 --features cli --bin substrate_probe

doc-link-check:
	cargo run --quiet -p xtask -- workflow doc-link-check

test:
	cargo test --workspace --locked
	$(MAKE) --no-print-directory authoritative-replay-cli-tests

authoritative-replay-cli-tests:
	cargo test -p dpw4 --features cli --test precision_authoritative_surface --locked

parser-tests:
	cargo run --quiet -p xtask -- workflow parser-tests

replay-tool-tests:
	cargo run --quiet -p xtask -- workflow replay-tool-tests

replay-tests:
	cargo run --quiet -p xtask -- workflow replay-tests

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
	DIFF_OUT="$$(PYTHONPATH=. python3 scripts/artifact_diff.py "$$BASELINE_A" "$$QUANTIZED_A")"; \
	FIRST_DIVERGENCE_FRAME="$$(printf '%s\n' "$$DIFF_OUT" | awk -F': ' '/^first_divergence_frame:/ {print $$2; exit}')"; \
	CLASSIFICATION="$$(printf '%s\n' "$$DIFF_OUT" | awk -F': ' '/^shape_class:/ {print $$2; exit}')"; \
	test -n "$$FIRST_DIVERGENCE_FRAME"; \
	test -n "$$CLASSIFICATION"; \
	echo "first_divergence_frame = $$FIRST_DIVERGENCE_FRAME"; \
	echo "classification = $$CLASSIFICATION"; \
	echo "baseline_invariant = true"

gate:
	cargo run --locked --release -p dpw4 --features cli --bin sig-util -- validate --mode quick

gate-full:
	cargo run --locked --release -p dpw4 --features cli --bin sig-util -- validate --mode full

ci-local:
	cargo run --quiet -p xtask -- workflow ci-local

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
	$(MAKE) --no-print-directory stream-purity

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
	cargo clean -p $(FW_PKG) -p dpw4

kani-gate:
	bash scripts/verify_kani.sh

kani-gate-tier2:
	RUN_HEAVY=1 bash scripts/verify_kani.sh

verify-repro:
	bash scripts/verify_release_repro.sh
