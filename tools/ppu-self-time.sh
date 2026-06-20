#!/bin/bash
# PPU-core cost attribution (L1.1 step 2) — behaviour-safe.
#
# The handoff suggested a `bench --profile` ablation that skips PPU
# fetch/shift/sprite work while keeping dot-advance + VBL/NMI + sprite-0. That
# can't be done without changing behaviour: sprite-0 hit and the overflow flag
# depend on the very pipeline being skipped, and v-register increments from the
# pipeline change mid-render $2007 reads — so the CPU would run a different
# instruction stream and the fps delta would be meaningless.
#
# Instead, attribute with a sampling profiler (macOS `sample`), which is fully
# non-perturbing — the emulator runs its exact normal path. The bottom-up
# "self time" table gives each function's true share. The PPU dot machine =
# Ppu::tick (incl. inlined render_pixel) + run_render_pipeline + load_tile_info
# + ppu_read_for + mirror_nt.
#
# Usage: tools/ppu-self-time.sh [rom] [seconds]
set -u
cd "$(dirname "$0")/.."
ROM="${1:-roms/SuperMarioBro.nes}"
SECS="${2:-8}"
BIN=target/release/fc
OUT=/tmp/fc-ppu-sample.txt

command -v sample >/dev/null || { echo "macOS 'sample' not found"; exit 1; }
[ -x "$BIN" ] || { echo "build first: cargo build -p fc-cli --release"; exit 1; }

"$BIN" bench "$ROM" --frames 40000 --warmup 60 >/dev/null 2>&1 &
PID=$!
sleep 2
sample "$PID" "$SECS" -f "$OUT" >/dev/null 2>&1
wait "$PID" 2>/dev/null

echo "=== self-time (bottom-up), $ROM, ${SECS}s ==="
awk '/Sort by top of stack/{f=1} f' "$OUT" | grep -E "fc_core|fc\)" | head -16
echo "--- PPU dot machine = Ppu::tick + run_render_pipeline + load_tile_info + ppu_read_for + mirror_nt"
