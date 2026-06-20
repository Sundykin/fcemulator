## ADDED Requirements

### Requirement: Per-instruction execution trace

The system SHALL provide a `fc trace <rom>` command that logs one line per
executed CPU instruction including PC, opcode bytes, disassembly, the A/X/Y/P/SP
registers, and a cycle counter. The number of instructions/frames traced MUST be
bounded by an option.

#### Scenario: Instruction trace output

- **WHEN** `fc trace roms/SuperMarioBro.nes --instrs 100000` runs
- **THEN** it emits 100000 trace lines, each with PC, opcode, disassembly,
  A/X/Y/P/SP and a cycle count

### Requirement: Mesen-aligned trace format

The trace SHALL be emitted in a format aligned with Mesen2 / Nintendulator trace
logs (field order and notation) so a line-by-line diff against a reference trace
is meaningful. The format MAY be selectable when more than one reference layout
is supported.

#### Scenario: Format matches reference layout

- **WHEN** the same program point is traced by `fc trace` and by Mesen2
- **THEN** the corresponding lines align field-for-field (PC, registers, cycle)
  closely enough to diff directly

### Requirement: Trace-diff convergence workflow

The system SHALL provide a way to diff an `fc trace` output against a reference
(Mesen2) trace and report the first diverging line, so accuracy work can locate
the exact instruction/cycle where behavior first differs.

#### Scenario: First divergence located

- **WHEN** an `fc trace` output is diffed against a Mesen2 reference trace that
  diverges at line N
- **THEN** the tool reports line N as the first divergence with both sides shown

### Requirement: Tracing is off the hot path by default

The trace hook in `fc-core` SHALL be gated so that normal emulation (no tracing)
incurs no measurable per-instruction cost from the trace facility.

#### Scenario: No tracing, no cost

- **WHEN** the emulator runs without tracing enabled
- **THEN** the benchmark fps is unchanged relative to before the trace hook was
  added (within benchmark tolerance)
