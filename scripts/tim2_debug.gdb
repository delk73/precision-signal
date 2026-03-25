# scripts/tim2_debug.gdb

set pagination off
set confirm off

file target/thumbv7em-none-eabihf/debug/replay-fw-f446

target extended-remote localhost:4242
monitor reset halt

printf "CPUID: "
x/wx 0xE000ED00

# Break at the wait loop line (works for your current build)
hbreak crates/replay-fw-f446/src/fw.rs:59
commands
  silent
  printf "\n--- Halted at Wait Loop ---\n"

  printf "CR1  : "
  x/wx 0x40000000

  printf "DIER : "
  x/wx 0x4000000C

  printf "SR   : "
  x/wx 0x40000010

  printf "CNT  : "
  x/wx 0x40000024

  printf "PSC  : "
  x/wx 0x40000028

  printf "ARR  : "
  x/wx 0x4000002C

  printf "ISER0: "
  x/wx 0xE000E100
end

printf "Starting execution...\n"
continue