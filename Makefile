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
STFLASH_SERIAL ?=
STFLASH_SERIAL_ARG = $(if $(strip $(STFLASH_SERIAL)),--serial $(strip $(STFLASH_SERIAL)),)
FLASH_HEAD := target/flash-head.bin
FLASH_FULL := target/flash-full.bin
SERIAL ?= /dev/ttyACM0
SIGNAL_BASELINE_CSV ?= baseline.csv
SIGNAL_OBSERVED_CSV ?= observed.csv
SIGNAL_FRAMES ?= 128
SIGNAL_PERTURB_FRAME ?= 50
SIGNAL_REPEAT_SECONDS ?=
FW_GATE_RESET_MODE ?= stlink
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
#   make flash-ur REPLAY_FW_FEATURES=signal-model-burst8
REPLAY_FW_FEATURES ?=
FW_FEATURES ?=

# Internal helper: expands to "--features a,b,c" or empty.
empty :=
space := $(empty) $(empty)
comma := ,
FW_FEATURES_EFFECTIVE = $(if $(strip $(FW_FEATURES)),$(strip $(FW_FEATURES)),$(strip $(REPLAY_FW_FEATURES)))
FW_FEATURES_ARG = $(if $(strip $(FW_FEATURES_EFFECTIVE)),--features $(subst $(space),$(comma),$(strip $(FW_FEATURES_EFFECTIVE))),)

.PHONY: help help-all help-demos help-firmware fixture-drift-check shell-check stflash-check bench-check fw fw-bin flash flash-verify flash-compare flash-ur flash-verify-ur flash-compare-ur demo-signal demo-signal-flash demo-signal-host-baseline demo-signal-host-perturb demo-signal-pi-baseline demo-signal-pi-perturb demo-signal-diff fw-capture-check fw-repeat-check rpl0-replay-check rpl0-replay-repeat-check rpl0-replay-repeat-auto fw-gate firmware-release-summary firmware-release-check fw-release-archive-current fw-release-archive release release-proof release-summary release-1.7.0 release-1.8.0 release-bundle release-bundle-check capture-demo-A capture-demo-B demo-captured-verify demo-captured-release demo-divergence demo-evidence-package replay-demo-audit debug-session tim2-smoke doc-link-check check-workspace test authoritative-replay-cli-tests parser-tests replay-tool-tests replay-tests gate gate-full ci-local conformance-audit kill-switch-audit stream-purity clean

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
	echo "  Active evidence:"
	echo "    make demo-evidence-package"
	echo "    make demo-divergence"
	echo "    make replay-demo-audit"
	echo "  Captured divergence support:"
	echo "    make demo-captured-verify"
	echo "    make demo-captured-release"
	echo "  Historical V2-V5 lifecycle flows are retained as scripts/fixtures,"
	echo "  not top-level Make operator targets."

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
	echo "    make fw-gate"
	echo "    make firmware-release-check"
	echo "    make fw-release-archive VERSION=<version>"
	echo "  Legacy / support timing capture:"
	echo "    make fw-capture-check"
	echo "    make fw-repeat-check"
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
	$(STFLASH) $(STFLASH_SERIAL_ARG) --reset write "$(FW_BIN)" "$(FLASH_ADDR)"

flash-ur: fw-bin stflash-check
	$(STFLASH) $(STFLASH_SERIAL_ARG) --connect-under-reset --freq=200K --reset write "$(FW_BIN)" "$(FLASH_ADDR)"

flash-verify: fw-bin stflash-check
	rm -f "$(FLASH_HEAD)"
	$(STFLASH) $(STFLASH_SERIAL_ARG) read "$(FLASH_HEAD)" "$(FLASH_ADDR)" 64
	test -s "$(FLASH_HEAD)"
	hexdump -C "$(FLASH_HEAD)" | sed -n '1,4p'
	$(PYTHON) scripts/check_flash_vectors.py "$(FLASH_HEAD)"

flash-verify-ur: fw-bin stflash-check
	rm -f "$(FLASH_HEAD)"
	$(STFLASH) $(STFLASH_SERIAL_ARG) --connect-under-reset --freq=200K read "$(FLASH_HEAD)" "$(FLASH_ADDR)" 64
	test -s "$(FLASH_HEAD)"
	hexdump -C "$(FLASH_HEAD)" | sed -n '1,4p'
	$(PYTHON) scripts/check_flash_vectors.py "$(FLASH_HEAD)"

flash-compare: fw-bin stflash-check
	bash scripts/compare_flash_image.sh --label flash-compare --stflash "$(STFLASH)" $(STFLASH_SERIAL_ARG) --addr "$(FLASH_ADDR)" --image "$(FW_BIN)" --out "$(FLASH_FULL)"

flash-compare-ur: fw-bin stflash-check
	bash scripts/compare_flash_image.sh --label flash-compare-ur --stflash "$(STFLASH)" $(STFLASH_SERIAL_ARG) --addr "$(FLASH_ADDR)" --image "$(FW_BIN)" --out "$(FLASH_FULL)" --under-reset

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

replay-demo-audit: shell-check
	$(PYTHON) scripts/replay_demo_audit.py

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
