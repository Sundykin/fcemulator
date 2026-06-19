//! 6502 disassembler (full 256-entry table incl. common unofficial opcodes).

use crate::bus::Bus;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Imp,
    Acc,
    Imm,
    Zp,
    ZpX,
    ZpY,
    Abs,
    AbsX,
    AbsY,
    Ind,
    IzX,
    IzY,
    Rel,
}

impl Mode {
    fn len(self) -> u16 {
        match self {
            Mode::Imp | Mode::Acc => 1,
            Mode::Imm | Mode::Zp | Mode::ZpX | Mode::ZpY | Mode::IzX | Mode::IzY | Mode::Rel => 2,
            Mode::Abs | Mode::AbsX | Mode::AbsY | Mode::Ind => 3,
        }
    }
}

/// Returns `(mnemonic, addressing mode)` for an opcode.
pub fn decode(op: u8) -> (&'static str, Mode) {
    use Mode::*;
    match op {
        0x00 => ("BRK", Imp), 0x01 => ("ORA", IzX), 0x05 => ("ORA", Zp), 0x06 => ("ASL", Zp),
        0x08 => ("PHP", Imp), 0x09 => ("ORA", Imm), 0x0A => ("ASL", Acc), 0x0D => ("ORA", Abs),
        0x0E => ("ASL", Abs), 0x10 => ("BPL", Rel), 0x11 => ("ORA", IzY), 0x15 => ("ORA", ZpX),
        0x16 => ("ASL", ZpX), 0x18 => ("CLC", Imp), 0x19 => ("ORA", AbsY), 0x1D => ("ORA", AbsX),
        0x1E => ("ASL", AbsX), 0x20 => ("JSR", Abs), 0x21 => ("AND", IzX), 0x24 => ("BIT", Zp),
        0x25 => ("AND", Zp), 0x26 => ("ROL", Zp), 0x28 => ("PLP", Imp), 0x29 => ("AND", Imm),
        0x2A => ("ROL", Acc), 0x2C => ("BIT", Abs), 0x2D => ("AND", Abs), 0x2E => ("ROL", Abs),
        0x30 => ("BMI", Rel), 0x31 => ("AND", IzY), 0x35 => ("AND", ZpX), 0x36 => ("ROL", ZpX),
        0x38 => ("SEC", Imp), 0x39 => ("AND", AbsY), 0x3D => ("AND", AbsX), 0x3E => ("ROL", AbsX),
        0x40 => ("RTI", Imp), 0x41 => ("EOR", IzX), 0x45 => ("EOR", Zp), 0x46 => ("LSR", Zp),
        0x48 => ("PHA", Imp), 0x49 => ("EOR", Imm), 0x4A => ("LSR", Acc), 0x4C => ("JMP", Abs),
        0x4D => ("EOR", Abs), 0x4E => ("LSR", Abs), 0x50 => ("BVC", Rel), 0x51 => ("EOR", IzY),
        0x55 => ("EOR", ZpX), 0x56 => ("LSR", ZpX), 0x58 => ("CLI", Imp), 0x59 => ("EOR", AbsY),
        0x5D => ("EOR", AbsX), 0x5E => ("LSR", AbsX), 0x60 => ("RTS", Imp), 0x61 => ("ADC", IzX),
        0x65 => ("ADC", Zp), 0x66 => ("ROR", Zp), 0x68 => ("PLA", Imp), 0x69 => ("ADC", Imm),
        0x6A => ("ROR", Acc), 0x6C => ("JMP", Ind), 0x6D => ("ADC", Abs), 0x6E => ("ROR", Abs),
        0x70 => ("BVS", Rel), 0x71 => ("ADC", IzY), 0x75 => ("ADC", ZpX), 0x76 => ("ROR", ZpX),
        0x78 => ("SEI", Imp), 0x79 => ("ADC", AbsY), 0x7D => ("ADC", AbsX), 0x7E => ("ROR", AbsX),
        0x81 => ("STA", IzX), 0x84 => ("STY", Zp), 0x85 => ("STA", Zp), 0x86 => ("STX", Zp),
        0x88 => ("DEY", Imp), 0x8A => ("TXA", Imp), 0x8C => ("STY", Abs), 0x8D => ("STA", Abs),
        0x8E => ("STX", Abs), 0x90 => ("BCC", Rel), 0x91 => ("STA", IzY), 0x94 => ("STY", ZpX),
        0x95 => ("STA", ZpX), 0x96 => ("STX", ZpY), 0x98 => ("TYA", Imp), 0x99 => ("STA", AbsY),
        0x9A => ("TXS", Imp), 0x9D => ("STA", AbsX), 0xA0 => ("LDY", Imm), 0xA1 => ("LDA", IzX),
        0xA2 => ("LDX", Imm), 0xA4 => ("LDY", Zp), 0xA5 => ("LDA", Zp), 0xA6 => ("LDX", Zp),
        0xA8 => ("TAY", Imp), 0xA9 => ("LDA", Imm), 0xAA => ("TAX", Imp), 0xAC => ("LDY", Abs),
        0xAD => ("LDA", Abs), 0xAE => ("LDX", Abs), 0xB0 => ("BCS", Rel), 0xB1 => ("LDA", IzY),
        0xB4 => ("LDY", ZpX), 0xB5 => ("LDA", ZpX), 0xB6 => ("LDX", ZpY), 0xB8 => ("CLV", Imp),
        0xB9 => ("LDA", AbsY), 0xBA => ("TSX", Imp), 0xBC => ("LDY", AbsX), 0xBD => ("LDA", AbsX),
        0xBE => ("LDX", AbsY), 0xC0 => ("CPY", Imm), 0xC1 => ("CMP", IzX), 0xC4 => ("CPY", Zp),
        0xC5 => ("CMP", Zp), 0xC6 => ("DEC", Zp), 0xC8 => ("INY", Imp), 0xC9 => ("CMP", Imm),
        0xCA => ("DEX", Imp), 0xCC => ("CPY", Abs), 0xCD => ("CMP", Abs), 0xCE => ("DEC", Abs),
        0xD0 => ("BNE", Rel), 0xD1 => ("CMP", IzY), 0xD5 => ("CMP", ZpX), 0xD6 => ("DEC", ZpX),
        0xD8 => ("CLD", Imp), 0xD9 => ("CMP", AbsY), 0xDD => ("CMP", AbsX), 0xDE => ("DEC", AbsX),
        0xE0 => ("CPX", Imm), 0xE1 => ("SBC", IzX), 0xE4 => ("CPX", Zp), 0xE5 => ("SBC", Zp),
        0xE6 => ("INC", Zp), 0xE8 => ("INX", Imp), 0xE9 => ("SBC", Imm), 0xEA => ("NOP", Imp),
        0xEC => ("CPX", Abs), 0xED => ("SBC", Abs), 0xEE => ("INC", Abs), 0xF0 => ("BEQ", Rel),
        0xF1 => ("SBC", IzY), 0xF5 => ("SBC", ZpX), 0xF6 => ("INC", ZpX), 0xF8 => ("SED", Imp),
        0xF9 => ("SBC", AbsY), 0xFD => ("SBC", AbsX), 0xFE => ("INC", AbsX),
        // common unofficial
        0xEB => ("*SBC", Imm),
        0x1A | 0x3A | 0x5A | 0x7A | 0xDA | 0xFA => ("*NOP", Imp),
        0x80 | 0x82 | 0x89 | 0xC2 | 0xE2 => ("*NOP", Imm),
        0x04 | 0x44 | 0x64 => ("*NOP", Zp),
        0x14 | 0x34 | 0x54 | 0x74 | 0xD4 | 0xF4 => ("*NOP", ZpX),
        0x0C => ("*NOP", Abs),
        0x1C | 0x3C | 0x5C | 0x7C | 0xDC | 0xFC => ("*NOP", AbsX),
        0xA7 => ("*LAX", Zp), 0xB7 => ("*LAX", ZpY), 0xAF => ("*LAX", Abs), 0xBF => ("*LAX", AbsY),
        0xA3 => ("*LAX", IzX), 0xB3 => ("*LAX", IzY),
        0x87 => ("*SAX", Zp), 0x97 => ("*SAX", ZpY), 0x8F => ("*SAX", Abs), 0x83 => ("*SAX", IzX),
        _ => ("???", Imp),
    }
}

/// Disassemble one instruction at `addr`. Returns the text and instruction size.
pub fn disassemble_at(bus: &Bus, addr: u16) -> (String, u16) {
    let op = bus.peek(addr);
    let (name, mode) = decode(op);
    let b1 = bus.peek(addr.wrapping_add(1));
    let b2 = bus.peek(addr.wrapping_add(2));
    let operand = match mode {
        Mode::Imp => String::new(),
        Mode::Acc => "A".to_string(),
        Mode::Imm => format!("#${:02X}", b1),
        Mode::Zp => format!("${:02X}", b1),
        Mode::ZpX => format!("${:02X},X", b1),
        Mode::ZpY => format!("${:02X},Y", b1),
        Mode::Abs => format!("${:04X}", u16::from_le_bytes([b1, b2])),
        Mode::AbsX => format!("${:04X},X", u16::from_le_bytes([b1, b2])),
        Mode::AbsY => format!("${:04X},Y", u16::from_le_bytes([b1, b2])),
        Mode::Ind => format!("(${:04X})", u16::from_le_bytes([b1, b2])),
        Mode::IzX => format!("(${:02X},X)", b1),
        Mode::IzY => format!("(${:02X}),Y", b1),
        Mode::Rel => {
            let target = addr.wrapping_add(2).wrapping_add((b1 as i8) as u16);
            format!("${:04X}", target)
        }
    };
    let text = if operand.is_empty() {
        format!("${:04X}: {}", addr, name)
    } else {
        format!("${:04X}: {} {}", addr, name, operand)
    };
    (text, mode.len())
}

/// Disassemble `count` instructions starting at `start`.
pub fn disassemble_range(bus: &Bus, start: u16, count: usize) -> Vec<String> {
    let mut out = Vec::with_capacity(count);
    let mut addr = start;
    for _ in 0..count {
        let (text, size) = disassemble_at(bus, addr);
        out.push(text);
        addr = addr.wrapping_add(size.max(1));
    }
    out
}
