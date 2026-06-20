#!/usr/bin/env python3
"""trace-diff — locate the first divergence between an `fc trace` output and a
reference (Mesen2 / Nintendulator) trace.

Both inputs are nestest/Nintendulator-style lines:

    C5F5  A2 00     LDX #$00     A:00 X:00 Y:00 P:24 SP:FD PPU:261,  9 CYC:3

Usage:
    fc trace <rom> --entry C000 --instrs 5003 > mine.log
    tools/trace-diff.py mine.log reference.log            # compare PC + registers
    tools/trace-diff.py --full mine.log reference.log     # also compare PPU + CYC

By default only PC and the CPU registers (A/X/Y/P/SP) are compared, since the
reset-cycle convention (nestest.log starts at CYC:7) and PPU dot alignment differ
harmlessly between tools. Use --full to demand byte-exact PPU/CYC parity too.
"""
import re
import sys

REG = re.compile(r"^([0-9A-Fa-f]{4})\b.*?(A:[0-9A-Fa-f]{2} X:[0-9A-Fa-f]{2} "
                 r"Y:[0-9A-Fa-f]{2} P:[0-9A-Fa-f]{2} SP:[0-9A-Fa-f]{2})"
                 r"(?:.*?(PPU:\s*\d+,\s*\d+\s+CYC:\d+))?")


def key(line, full):
    m = REG.search(line)
    if not m:
        return None
    return (m.group(1).upper(), m.group(2), (m.group(3) or "") if full else "")


def main():
    args = [a for a in sys.argv[1:] if a != "--full"]
    full = "--full" in sys.argv
    if len(args) != 2:
        print(__doc__)
        sys.exit(2)
    a_lines = open(args[0]).read().splitlines()
    b_lines = open(args[1]).read().splitlines()
    a = [(i, l, key(l, full)) for i, l in enumerate(a_lines, 1)]
    b = [(i, l, key(l, full)) for i, l in enumerate(b_lines, 1)]
    a = [x for x in a if x[2] is not None]
    b = [x for x in b if x[2] is not None]
    n = min(len(a), len(b))
    for idx in range(n):
        if a[idx][2] != b[idx][2]:
            print(f"DIVERGE at matched instruction #{idx + 1}")
            print(f"  fc  (line {a[idx][0]}): {a[idx][1]}")
            print(f"  ref (line {b[idx][0]}): {b[idx][1]}")
            print(f"\n  {idx} instructions matched before divergence.")
            sys.exit(1)
    if len(a) != len(b):
        print(f"OK for {n} instructions, then one side ended "
              f"(fc={len(a)}, ref={len(b)}).")
        sys.exit(0)
    print(f"IDENTICAL across {n} instructions "
          f"({'PC+regs+PPU+CYC' if full else 'PC+regs'}).")


if __name__ == "__main__":
    main()
