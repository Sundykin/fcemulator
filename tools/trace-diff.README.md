# Trace diffing fc against Mesen2

The accuracy weapon for the optimization roadmap (`docs/模拟器优化计划.md`, Layer 3):
run the same program point in `fc trace` and in Mesen2, then `tools/trace-diff.py`
finds the first instruction where the two diverge.

## 1. Produce the fc trace

```sh
fc trace <rom.nes> --entry C000 --instrs 100000 > /tmp/fc.log     # nestest-style
# or bound by frames:
fc trace <rom.nes> --frames 10 > /tmp/fc.log
```

Each line is `PC  bytes  DISASM  A:.. X:.. Y:.. P:.. SP:.. PPU:sl,dot CYC:cyc`.

## 2. Produce the Mesen2 reference trace

In Mesen2: **Debug → Trace Logger**. Enable it, set **Format** to a string that
matches fc's layout, then run the same program point and **Save** the log:

```
[PC]  [ByteCode]  [Disassembly]  A:[A] X:[X] Y:[Y] P:[P] SP:[SP] PPU:[Scanline],[Cycle] CYC:[CycleCount]
```

(Use the same ROM and the same entry point; for nestest start at `C000`.)

## 3. Diff

```sh
tools/trace-diff.py /tmp/fc.log /tmp/mesen.log          # PC + CPU registers only
tools/trace-diff.py --full /tmp/fc.log /tmp/mesen.log   # also demand PPU + CYC parity
```

- Default compares **PC + A/X/Y/P/SP** — robust to the reset-cycle convention
  (`nestest.log` starts at `CYC:7`) and minor PPU-dot alignment.
- `--full` additionally requires the `PPU:sl,dot` and `CYC` columns to match,
  i.e. full cycle parity — the bar for Layer-3 sign-off.

## Validation baseline

`fc trace nes-test-roms/other/nestest.nes --entry C000 --instrs 5003` already
matches `nes-test-roms/other/nestest.log` on PC + registers for all 5003
instructions — proving the CPU register model and the trace format are correct.
