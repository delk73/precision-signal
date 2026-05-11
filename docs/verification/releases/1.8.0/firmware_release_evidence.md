# Firmware Release Evidence (1.8.0)

REPLAY_RUN=artifacts/run.bin
REPLAY_REPEAT_DIR=artifacts/replay_runs

## capture hash check
f79e71d6ed645f6bc9f7c3d2b4a8980e0a8cee11cc17082e649966ffba20e765  artifacts/run.bin

## repeat manifest
# rpl0 repeat_capture manifest
contract=rpl0
serial=/dev/ttyACM0 runs=5
reset_mode=manual
timeout_seconds=10.0
signal_model=phase8
# fields: contract run file bytes sha256 rows status
contract=rpl0 run=01 file=run_01.bin bytes=160243 sha256=f79e71d6ed645f6bc9f7c3d2b4a8980e0a8cee11cc17082e649966ffba20e765 rows=0 status=PASS
contract=rpl0 run=02 file=run_02.bin bytes=160243 sha256=f79e71d6ed645f6bc9f7c3d2b4a8980e0a8cee11cc17082e649966ffba20e765 rows=0 status=PASS
contract=rpl0 run=03 file=run_03.bin bytes=160243 sha256=f79e71d6ed645f6bc9f7c3d2b4a8980e0a8cee11cc17082e649966ffba20e765 rows=0 status=PASS
contract=rpl0 run=04 file=run_04.bin bytes=160243 sha256=f79e71d6ed645f6bc9f7c3d2b4a8980e0a8cee11cc17082e649966ffba20e765 rows=0 status=PASS
contract=rpl0 run=05 file=run_05.bin bytes=160243 sha256=f79e71d6ed645f6bc9f7c3d2b4a8980e0a8cee11cc17082e649966ffba20e765 rows=0 status=PASS
