#!/usr/bin/env python3
"""External cycle-parity gate: diff `fc trace` against the golden nestest.log.

nestest.log is the Nintendulator/Mesen-class reference trace. This checks two
things over the first 5003 instructions:

  1. PC + A/X/Y/P/SP are byte-identical (CPU model + behaviour correct).
  2. The CYC and PPU(scanline,dot) columns differ from the reference only by a
     *constant* offset — i.e. fc advances the CPU and PPU clocks at exactly the
     reference rate, with zero drift. (The offset itself is just the reset
     zero-point convention: Nintendulator starts at CYC:7 / PPU:0,21, fc at
     CYC:0 / PPU:261,0.) A single distinct offset value == cycle-exact parity;
     more than one == a real timing divergence.

Usage:
  tools/nestest-parity.py [fc_binary] [nestest.nes] [nestest.log]
Exit non-zero on any parity failure, so it can gate CI / refactors.
"""
import re
import subprocess
import sys

FC = sys.argv[1] if len(sys.argv) > 1 else "target/release/fc"
ROM = sys.argv[2] if len(sys.argv) > 2 else "nes-test-roms/other/nestest.nes"
REF = sys.argv[3] if len(sys.argv) > 3 else "nes-test-roms/other/nestest.log"
N = 5003


def parse(line):
    pc = line[:4]
    regs = re.search(r"A:(\w\w) X:(\w\w) Y:(\w\w) P:(\w\w) SP:(\w\w)", line)
    cyc = int(re.search(r"CYC:(\d+)", line).group(1))
    p = re.search(r"PPU:\s*(\d+),\s*(\d+)", line)
    sl, dot = int(p.group(1)), int(p.group(2))
    # Normalise the pre-render line label (fc=261, Nintendulator=0/-1) to a
    # single absolute dot-in-frame axis so a constant frame offset collapses out.
    s = -1 if sl == 261 else sl
    return pc, regs.groups(), cyc, s * 341 + dot


def main():
    fc_txt = subprocess.run(
        [FC, "trace", ROM, "--entry", "C000", "--instrs", str(N)],
        capture_output=True, text=True, check=True,
    ).stdout.splitlines()
    ref_txt = [l for l in open(REF) if l.strip()]

    fc = [parse(l) for l in fc_txt if l.strip()][:N]
    ref = [parse(l) for l in ref_txt][:N]
    if len(fc) < N or len(ref) < N:
        print(f"FAIL: not enough lines (fc={len(fc)}, ref={len(ref)}, need {N})")
        return 1

    pc_reg_ok = all(a[0] == b[0] and a[1] == b[1] for a, b in zip(fc, ref))
    cyc_off = {b[2] - a[2] for a, b in zip(fc, ref)}
    ppu_off = {b[3] - a[3] for a, b in zip(fc, ref)}

    print(f"instructions compared : {N}")
    print(f"PC + A/X/Y/P/SP exact : {'OK' if pc_reg_ok else 'FAIL'}")
    print(f"CYC offset (ref-fc)   : {sorted(cyc_off)}  ({'constant' if len(cyc_off)==1 else 'DRIFTS'})")
    print(f"PPU dot offset        : {sorted(ppu_off)}  ({'constant' if len(ppu_off)==1 else 'DRIFTS'})")

    ok = pc_reg_ok and len(cyc_off) == 1 and len(ppu_off) == 1
    print("RESULT:", "CYCLE-EXACT PARITY (modulo reset convention)" if ok else "PARITY FAILURE")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
