//! Ricoh 2A03 CPU (NMOS 6502 without decimal mode).
//!
//! The core is *cycle-driven*: every real bus read/write ticks the rest of the
//! system (`Bus::tick` → PPU ×3, APU ×1), so PPU/APU stay in lock-step with the
//! CPU at sub-instruction granularity. Cycle counts emerge from the number of
//! bus accesses plus explicit internal ("dummy") cycles where the 6502 has them.

use crate::bus::Bus;
use serde::{Deserialize, Serialize};

pub mod flags {
    pub const C: u8 = 0x01;
    pub const Z: u8 = 0x02;
    pub const I: u8 = 0x04;
    pub const D: u8 = 0x08;
    pub const B: u8 = 0x10;
    pub const U: u8 = 0x20;
    pub const V: u8 = 0x40;
    pub const N: u8 = 0x80;
}
use flags::*;

pub const NMI_VECTOR: u16 = 0xFFFA;
pub const RESET_VECTOR: u16 = 0xFFFC;
pub const IRQ_VECTOR: u16 = 0xFFFE;
const STACK_BASE: u16 = 0x0100;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
enum InterruptKind {
    Nmi,
    Irq,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
struct InterruptPoll {
    nmi: bool,
    irq: bool,
    i: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cpu {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub sp: u8,
    pub pc: u16,
    pub p: u8,
    pub cycles: u64,
    pub nmi_count: u64,
    pub trace: bool,
    /// CPU-internal NMI edge detector output. The PPU/bus owns the raw edge;
    /// the CPU samples it once per CPU cycle and holds it until a poll point.
    nmi_pending: bool,
    /// CPU-internal IRQ level detector output sampled at the end of the
    /// previous CPU cycle.
    irq_sample: bool,
    last_poll: InterruptPoll,
    pending_interrupt: Option<InterruptKind>,
    suppress_instruction_poll: bool,
}

impl Default for Cpu {
    fn default() -> Self {
        Cpu::new()
    }
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            a: 0,
            x: 0,
            y: 0,
            sp: 0xFD,
            pc: 0,
            p: 0x24,
            cycles: 0,
            nmi_count: 0,
            trace: false,
            nmi_pending: false,
            irq_sample: false,
            last_poll: InterruptPoll::default(),
            pending_interrupt: None,
            suppress_instruction_poll: false,
        }
    }

    pub fn power_on(&mut self, bus: &mut Bus) {
        let lo = bus.read(RESET_VECTOR);
        let hi = bus.read(RESET_VECTOR + 1);
        self.pc = u16::from_le_bytes([lo, hi]);
        self.sp = 0xFD;
        self.p = 0x24;
        self.cycles = 0;
        self.nmi_pending = false;
        self.irq_sample = false;
        self.last_poll = InterruptPoll::default();
        self.pending_interrupt = None;
        self.suppress_instruction_poll = false;
    }

    pub fn reset(&mut self, bus: &mut Bus) {
        let lo = bus.read(RESET_VECTOR);
        let hi = bus.read(RESET_VECTOR + 1);
        self.pc = u16::from_le_bytes([lo, hi]);
        self.sp = self.sp.wrapping_sub(3);
        self.p |= I;
        self.nmi_pending = false;
        self.irq_sample = false;
        self.last_poll = InterruptPoll::default();
        self.pending_interrupt = None;
        self.suppress_instruction_poll = false;
    }

    // --------------------------------------------------------- bus accessors

    #[inline]
    fn begin_cycle(&mut self) {
        self.last_poll = InterruptPoll {
            nmi: self.nmi_pending,
            irq: self.irq_sample,
            i: self.p & I != 0,
        };
    }

    #[inline]
    fn end_cycle(&mut self, bus: &mut Bus) {
        if bus.poll_nmi() {
            self.nmi_pending = true;
        }
        self.irq_sample = bus.irq_line();
    }

    /// Run any DMA the arbiter wants to perform before a halt-able CPU cycle.
    /// Each stolen cycle still advances PPU/APU and samples the NMI/IRQ lines,
    /// but does NOT open a new instruction interrupt-poll point (no
    /// `begin_cycle`), so an RDY-held read never counts as a completed micro-op.
    /// `addr` is the address the held read drives — DMC repeats it as a dummy
    /// read (the $4016/$2007 extra-read behaviour).
    #[inline]
    fn pump_dma(&mut self, bus: &mut Bus, addr: u16) {
        while bus.dma_halt_pending() {
            bus.tick();
            self.cycles += 1;
            bus.dma_clock(addr);
            self.end_cycle(bus);
        }
    }

    #[inline]
    fn rd(&mut self, bus: &mut Bus, addr: u16) -> u8 {
        self.pump_dma(bus, addr); // reads are RDY-halt-able
        loop {
            self.begin_cycle();
            bus.tick();
            self.cycles += 1;
            if bus.dma_halt_wanted() {
                bus.dma_clock(addr);
                self.end_cycle(bus);
                self.pump_dma(bus, addr);
                continue;
            }
            let value = bus.read(addr);
            self.end_cycle(bus);
            return value;
        }
    }

    #[inline]
    fn wr(&mut self, bus: &mut Bus, addr: u16, value: u8) {
        // Writes are NOT halt-able: the write completes, DMA waits a cycle.
        self.begin_cycle();
        bus.tick();
        self.cycles += 1;
        bus.write(addr, value);
        self.end_cycle(bus);
    }

    /// An internal cycle with no external bus effect (still ticks the system).
    #[inline]
    fn io(&mut self, bus: &mut Bus) {
        // The 6502 still drives the address bus on internal cycles, so RDY can
        // halt them. There is no meaningful data address, so DMC repeats PC.
        self.pump_dma(bus, self.pc);
        loop {
            self.begin_cycle();
            bus.tick();
            self.cycles += 1;
            if bus.dma_halt_wanted() {
                bus.dma_clock(self.pc);
                self.end_cycle(bus);
                self.pump_dma(bus, self.pc);
                continue;
            }
            self.end_cycle(bus);
            return;
        }
    }

    fn fetch(&mut self, bus: &mut Bus) -> u8 {
        let v = self.rd(bus, self.pc);
        self.pc = self.pc.wrapping_add(1);
        v
    }

    fn fetch16(&mut self, bus: &mut Bus) -> u16 {
        let lo = self.fetch(bus) as u16;
        let hi = self.fetch(bus) as u16;
        (hi << 8) | lo
    }

    fn push(&mut self, bus: &mut Bus, v: u8) {
        self.wr(bus, STACK_BASE + self.sp as u16, v);
        self.sp = self.sp.wrapping_sub(1);
    }
    fn pull(&mut self, bus: &mut Bus) -> u8 {
        self.sp = self.sp.wrapping_add(1);
        self.rd(bus, STACK_BASE + self.sp as u16)
    }
    fn push16(&mut self, bus: &mut Bus, v: u16) {
        self.push(bus, (v >> 8) as u8);
        self.push(bus, v as u8);
    }
    fn pull16(&mut self, bus: &mut Bus) -> u16 {
        let lo = self.pull(bus) as u16;
        let hi = self.pull(bus) as u16;
        (hi << 8) | lo
    }

    // ------------------------------------------------------------- flags util

    #[inline]
    fn set_flag(&mut self, flag: u8, on: bool) {
        if on {
            self.p |= flag;
        } else {
            self.p &= !flag;
        }
    }
    #[inline]
    fn set_zn(&mut self, v: u8) {
        self.set_flag(Z, v == 0);
        self.set_flag(N, v & 0x80 != 0);
    }

    // ----------------------------------------------------------- interrupts

    fn interrupt(&mut self, bus: &mut Bus, vector: u16, brk: bool, vector_after_status: bool) {
        self.io(bus);
        self.io(bus);
        self.push16(bus, self.pc);
        let mut selected_vector = vector;
        if !vector_after_status {
            selected_vector = self.select_interrupt_vector(vector);
        }
        let mut status = self.p | U;
        if brk {
            status |= B;
        } else {
            status &= !B;
        }
        self.push(bus, status);
        self.set_flag(I, true);
        if vector_after_status {
            selected_vector = self.select_interrupt_vector(vector);
        }
        let lo = self.rd(bus, selected_vector) as u16;
        let hi = self.rd(bus, selected_vector + 1) as u16;
        self.pc = (hi << 8) | lo;
        self.suppress_instruction_poll = true;
    }

    fn brk(&mut self, bus: &mut Bus) {
        self.fetch(bus); // padding byte
        self.push16(bus, self.pc);
        self.push(bus, self.p | U | B);
        self.set_flag(I, true);
        let vector = self.select_interrupt_vector(IRQ_VECTOR);
        let lo = self.rd(bus, vector) as u16;
        let hi = self.rd(bus, vector + 1) as u16;
        self.pc = (hi << 8) | lo;
        self.suppress_instruction_poll = true;
    }

    fn nmi(&mut self, bus: &mut Bus) {
        self.nmi_count += 1;
        self.interrupt(bus, NMI_VECTOR, false, false);
    }
    fn irq(&mut self, bus: &mut Bus) {
        self.interrupt(bus, IRQ_VECTOR, false, true);
    }

    fn service_pending_interrupt(&mut self, bus: &mut Bus) -> bool {
        match self.pending_interrupt.take() {
            Some(InterruptKind::Nmi) => {
                self.nmi(bus);
                true
            }
            Some(InterruptKind::Irq) => {
                self.irq(bus);
                true
            }
            None => false,
        }
    }

    // --------------------------------------------------------------- step

    /// Execute one instruction (or service a pending interrupt).
    pub fn step(&mut self, bus: &mut Bus) {
        if self.service_pending_interrupt(bus) {
            return;
        }

        let pc = self.pc;
        let opcode = self.fetch(bus);
        if self.trace {
            eprintln!(
                "{:04X}  {:02X}  A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} CYC:{}",
                pc, opcode, self.a, self.x, self.y, self.p, self.sp, self.cycles
            );
        }
        self.execute(bus, opcode);
        self.poll_interrupts();
    }

    // ----------------------------------------------------- addressing modes
    // Each returns the effective address. For indexed read modes, the
    // page-cross dummy read is performed here when `penalty` is true.

    fn zp(&mut self, bus: &mut Bus) -> u16 {
        self.fetch(bus) as u16
    }
    fn zpx(&mut self, bus: &mut Bus) -> u16 {
        let base = self.fetch(bus);
        let _ = self.rd(bus, base as u16); // dummy read of the zp base
        base.wrapping_add(self.x) as u16
    }
    fn zpy(&mut self, bus: &mut Bus) -> u16 {
        let base = self.fetch(bus);
        let _ = self.rd(bus, base as u16);
        base.wrapping_add(self.y) as u16
    }
    fn abs(&mut self, bus: &mut Bus) -> u16 {
        self.fetch16(bus)
    }
    /// The 6502 dummy read on indexed addressing reads the address formed with
    /// the low byte added but the high-byte carry NOT yet applied. This read is
    /// real (it can hit PPU/APU registers with side effects).
    fn indexed_dummy(&mut self, bus: &mut Bus, base: u16, addr: u16) {
        let wrong = (base & 0xFF00) | (addr & 0x00FF);
        let _ = self.rd(bus, wrong);
    }
    fn abx(&mut self, bus: &mut Bus, penalty: bool) -> u16 {
        let base = self.fetch16(bus);
        let addr = base.wrapping_add(self.x as u16);
        if !penalty || page_crossed(base, addr) {
            self.indexed_dummy(bus, base, addr);
        }
        addr
    }
    fn aby(&mut self, bus: &mut Bus, penalty: bool) -> u16 {
        let base = self.fetch16(bus);
        let addr = base.wrapping_add(self.y as u16);
        if !penalty || page_crossed(base, addr) {
            self.indexed_dummy(bus, base, addr);
        }
        addr
    }
    fn izx(&mut self, bus: &mut Bus) -> u16 {
        let t = self.fetch(bus);
        let _ = self.rd(bus, t as u16); // dummy read of the zp pointer base
        let ptr = t.wrapping_add(self.x);
        let lo = self.rd(bus, ptr as u16) as u16;
        let hi = self.rd(bus, ptr.wrapping_add(1) as u16) as u16;
        (hi << 8) | lo
    }
    fn izy(&mut self, bus: &mut Bus, penalty: bool) -> u16 {
        let t = self.fetch(bus);
        let lo = self.rd(bus, t as u16) as u16;
        let hi = self.rd(bus, t.wrapping_add(1) as u16) as u16;
        let base = (hi << 8) | lo;
        let addr = base.wrapping_add(self.y as u16);
        if !penalty || page_crossed(base, addr) {
            self.indexed_dummy(bus, base, addr);
        }
        addr
    }

    // ------------------------------------------------------------ operations

    fn lda(&mut self, v: u8) {
        self.a = v;
        self.set_zn(v);
    }
    fn ldx(&mut self, v: u8) {
        self.x = v;
        self.set_zn(v);
    }
    fn ldy(&mut self, v: u8) {
        self.y = v;
        self.set_zn(v);
    }
    fn and(&mut self, v: u8) {
        self.a &= v;
        let a = self.a;
        self.set_zn(a);
    }
    fn ora(&mut self, v: u8) {
        self.a |= v;
        let a = self.a;
        self.set_zn(a);
    }
    fn eor(&mut self, v: u8) {
        self.a ^= v;
        let a = self.a;
        self.set_zn(a);
    }
    fn adc(&mut self, v: u8) {
        let a = self.a as u16;
        let m = v as u16;
        let c = (self.p & C) as u16;
        let sum = a + m + c;
        let r = sum as u8;
        self.set_flag(C, sum > 0xFF);
        self.set_flag(V, (!(self.a ^ v) & (self.a ^ r) & 0x80) != 0);
        self.a = r;
        self.set_zn(r);
    }
    fn sbc(&mut self, v: u8) {
        self.adc(v ^ 0xFF);
    }
    fn compare(&mut self, reg: u8, v: u8) {
        let r = reg.wrapping_sub(v);
        self.set_flag(C, reg >= v);
        self.set_zn(r);
    }
    fn bit(&mut self, v: u8) {
        self.set_flag(Z, self.a & v == 0);
        self.set_flag(N, v & 0x80 != 0);
        self.set_flag(V, v & 0x40 != 0);
    }

    fn queue_interrupt(&mut self, kind: InterruptKind) {
        if self.pending_interrupt != Some(InterruptKind::Nmi) {
            self.pending_interrupt = Some(kind);
        }
    }

    fn accept_interrupt_poll(&mut self, poll: InterruptPoll) {
        if self.nmi_pending {
            self.nmi_pending = false;
            self.queue_interrupt(InterruptKind::Nmi);
        } else if poll.irq && !poll.i {
            self.queue_interrupt(InterruptKind::Irq);
        }
    }

    fn accept_exact_interrupt_poll(&mut self, poll: InterruptPoll) {
        if poll.nmi {
            self.nmi_pending = false;
            self.queue_interrupt(InterruptKind::Nmi);
        } else if poll.irq && !poll.i {
            self.queue_interrupt(InterruptKind::Irq);
        }
    }

    fn poll_interrupts(&mut self) {
        if self.suppress_instruction_poll {
            self.suppress_instruction_poll = false;
            return;
        }
        self.accept_interrupt_poll(self.last_poll);
    }

    fn select_interrupt_vector(&mut self, vector: u16) -> u16 {
        if vector != NMI_VECTOR && self.nmi_pending {
            self.nmi_pending = false;
            self.nmi_count += 1;
            NMI_VECTOR
        } else {
            vector
        }
    }

    fn asl_val(&mut self, v: u8) -> u8 {
        self.set_flag(C, v & 0x80 != 0);
        let r = v << 1;
        self.set_zn(r);
        r
    }
    fn lsr_val(&mut self, v: u8) -> u8 {
        self.set_flag(C, v & 0x01 != 0);
        let r = v >> 1;
        self.set_zn(r);
        r
    }
    fn rol_val(&mut self, v: u8) -> u8 {
        let carry = self.p & C;
        self.set_flag(C, v & 0x80 != 0);
        let r = (v << 1) | carry;
        self.set_zn(r);
        r
    }
    fn ror_val(&mut self, v: u8) -> u8 {
        let carry = (self.p & C) << 7;
        self.set_flag(C, v & 0x01 != 0);
        let r = (v >> 1) | carry;
        self.set_zn(r);
        r
    }

    /// Read-modify-write helper (dummy write of the original value included).
    fn rmw<F: FnOnce(&mut Self, u8) -> u8>(&mut self, bus: &mut Bus, addr: u16, f: F) {
        let v = self.rd(bus, addr);
        self.wr(bus, addr, v); // dummy write-back
        let r = f(self, v);
        self.wr(bus, addr, r);
    }

    fn branch(&mut self, bus: &mut Bus, cond: bool) {
        let off = self.fetch(bus) as i8;
        if cond {
            let branch_poll = self.last_poll;
            self.io(bus); // taken-branch internal cycle
            let old = self.pc;
            self.pc = (self.pc as i32 + off as i32) as u16;
            if page_crossed(old, self.pc) {
                self.io(bus);
            } else {
                self.accept_exact_interrupt_poll(branch_poll);
                self.suppress_instruction_poll = true;
            }
        }
    }

    // --------------------------------------------------------- dispatch

    fn execute(&mut self, bus: &mut Bus, opcode: u8) {
        match opcode {
            // ---- loads ----
            0xA9 => {
                let v = self.fetch(bus);
                self.lda(v);
            }
            0xA5 => {
                let a = self.zp(bus);
                let v = self.rd(bus, a);
                self.lda(v);
            }
            0xB5 => {
                let a = self.zpx(bus);
                let v = self.rd(bus, a);
                self.lda(v);
            }
            0xAD => {
                let a = self.abs(bus);
                let v = self.rd(bus, a);
                self.lda(v);
            }
            0xBD => {
                let a = self.abx(bus, true);
                let v = self.rd(bus, a);
                self.lda(v);
            }
            0xB9 => {
                let a = self.aby(bus, true);
                let v = self.rd(bus, a);
                self.lda(v);
            }
            0xA1 => {
                let a = self.izx(bus);
                let v = self.rd(bus, a);
                self.lda(v);
            }
            0xB1 => {
                let a = self.izy(bus, true);
                let v = self.rd(bus, a);
                self.lda(v);
            }

            0xA2 => {
                let v = self.fetch(bus);
                self.ldx(v);
            }
            0xA6 => {
                let a = self.zp(bus);
                let v = self.rd(bus, a);
                self.ldx(v);
            }
            0xB6 => {
                let a = self.zpy(bus);
                let v = self.rd(bus, a);
                self.ldx(v);
            }
            0xAE => {
                let a = self.abs(bus);
                let v = self.rd(bus, a);
                self.ldx(v);
            }
            0xBE => {
                let a = self.aby(bus, true);
                let v = self.rd(bus, a);
                self.ldx(v);
            }

            0xA0 => {
                let v = self.fetch(bus);
                self.ldy(v);
            }
            0xA4 => {
                let a = self.zp(bus);
                let v = self.rd(bus, a);
                self.ldy(v);
            }
            0xB4 => {
                let a = self.zpx(bus);
                let v = self.rd(bus, a);
                self.ldy(v);
            }
            0xAC => {
                let a = self.abs(bus);
                let v = self.rd(bus, a);
                self.ldy(v);
            }
            0xBC => {
                let a = self.abx(bus, true);
                let v = self.rd(bus, a);
                self.ldy(v);
            }

            // ---- stores ----
            0x85 => {
                let a = self.zp(bus);
                self.wr(bus, a, self.a);
            }
            0x95 => {
                let a = self.zpx(bus);
                self.wr(bus, a, self.a);
            }
            0x8D => {
                let a = self.abs(bus);
                self.wr(bus, a, self.a);
            }
            0x9D => {
                let a = self.abx(bus, false);
                self.wr(bus, a, self.a);
            }
            0x99 => {
                let a = self.aby(bus, false);
                self.wr(bus, a, self.a);
            }
            0x81 => {
                let a = self.izx(bus);
                self.wr(bus, a, self.a);
            }
            0x91 => {
                let a = self.izy(bus, false);
                self.wr(bus, a, self.a);
            }
            0x86 => {
                let a = self.zp(bus);
                self.wr(bus, a, self.x);
            }
            0x96 => {
                let a = self.zpy(bus);
                self.wr(bus, a, self.x);
            }
            0x8E => {
                let a = self.abs(bus);
                self.wr(bus, a, self.x);
            }
            0x84 => {
                let a = self.zp(bus);
                self.wr(bus, a, self.y);
            }
            0x94 => {
                let a = self.zpx(bus);
                self.wr(bus, a, self.y);
            }
            0x8C => {
                let a = self.abs(bus);
                self.wr(bus, a, self.y);
            }

            // ---- transfers ----
            0xAA => {
                self.io(bus);
                self.x = self.a;
                let x = self.x;
                self.set_zn(x);
            }
            0xA8 => {
                self.io(bus);
                self.y = self.a;
                let y = self.y;
                self.set_zn(y);
            }
            0x8A => {
                self.io(bus);
                self.a = self.x;
                let a = self.a;
                self.set_zn(a);
            }
            0x98 => {
                self.io(bus);
                self.a = self.y;
                let a = self.a;
                self.set_zn(a);
            }
            0xBA => {
                self.io(bus);
                self.x = self.sp;
                let x = self.x;
                self.set_zn(x);
            }
            0x9A => {
                self.io(bus);
                self.sp = self.x;
            }

            // ---- stack ----
            0x48 => {
                self.io(bus);
                self.push(bus, self.a);
            }
            0x08 => {
                self.io(bus);
                self.push(bus, self.p | U | B);
            }
            0x68 => {
                self.io(bus);
                self.io(bus);
                let v = self.pull(bus);
                self.lda(v);
            }
            0x28 => {
                self.io(bus);
                self.io(bus);
                let v = self.pull(bus);
                self.p = (v & !B) | U;
            }

            // ---- logic / arithmetic ----
            0x29 => {
                let v = self.fetch(bus);
                self.and(v);
            }
            0x25 => {
                let a = self.zp(bus);
                let v = self.rd(bus, a);
                self.and(v);
            }
            0x35 => {
                let a = self.zpx(bus);
                let v = self.rd(bus, a);
                self.and(v);
            }
            0x2D => {
                let a = self.abs(bus);
                let v = self.rd(bus, a);
                self.and(v);
            }
            0x3D => {
                let a = self.abx(bus, true);
                let v = self.rd(bus, a);
                self.and(v);
            }
            0x39 => {
                let a = self.aby(bus, true);
                let v = self.rd(bus, a);
                self.and(v);
            }
            0x21 => {
                let a = self.izx(bus);
                let v = self.rd(bus, a);
                self.and(v);
            }
            0x31 => {
                let a = self.izy(bus, true);
                let v = self.rd(bus, a);
                self.and(v);
            }

            0x09 => {
                let v = self.fetch(bus);
                self.ora(v);
            }
            0x05 => {
                let a = self.zp(bus);
                let v = self.rd(bus, a);
                self.ora(v);
            }
            0x15 => {
                let a = self.zpx(bus);
                let v = self.rd(bus, a);
                self.ora(v);
            }
            0x0D => {
                let a = self.abs(bus);
                let v = self.rd(bus, a);
                self.ora(v);
            }
            0x1D => {
                let a = self.abx(bus, true);
                let v = self.rd(bus, a);
                self.ora(v);
            }
            0x19 => {
                let a = self.aby(bus, true);
                let v = self.rd(bus, a);
                self.ora(v);
            }
            0x01 => {
                let a = self.izx(bus);
                let v = self.rd(bus, a);
                self.ora(v);
            }
            0x11 => {
                let a = self.izy(bus, true);
                let v = self.rd(bus, a);
                self.ora(v);
            }

            0x49 => {
                let v = self.fetch(bus);
                self.eor(v);
            }
            0x45 => {
                let a = self.zp(bus);
                let v = self.rd(bus, a);
                self.eor(v);
            }
            0x55 => {
                let a = self.zpx(bus);
                let v = self.rd(bus, a);
                self.eor(v);
            }
            0x4D => {
                let a = self.abs(bus);
                let v = self.rd(bus, a);
                self.eor(v);
            }
            0x5D => {
                let a = self.abx(bus, true);
                let v = self.rd(bus, a);
                self.eor(v);
            }
            0x59 => {
                let a = self.aby(bus, true);
                let v = self.rd(bus, a);
                self.eor(v);
            }
            0x41 => {
                let a = self.izx(bus);
                let v = self.rd(bus, a);
                self.eor(v);
            }
            0x51 => {
                let a = self.izy(bus, true);
                let v = self.rd(bus, a);
                self.eor(v);
            }

            0x69 => {
                let v = self.fetch(bus);
                self.adc(v);
            }
            0x65 => {
                let a = self.zp(bus);
                let v = self.rd(bus, a);
                self.adc(v);
            }
            0x75 => {
                let a = self.zpx(bus);
                let v = self.rd(bus, a);
                self.adc(v);
            }
            0x6D => {
                let a = self.abs(bus);
                let v = self.rd(bus, a);
                self.adc(v);
            }
            0x7D => {
                let a = self.abx(bus, true);
                let v = self.rd(bus, a);
                self.adc(v);
            }
            0x79 => {
                let a = self.aby(bus, true);
                let v = self.rd(bus, a);
                self.adc(v);
            }
            0x61 => {
                let a = self.izx(bus);
                let v = self.rd(bus, a);
                self.adc(v);
            }
            0x71 => {
                let a = self.izy(bus, true);
                let v = self.rd(bus, a);
                self.adc(v);
            }

            0xE9 | 0xEB => {
                let v = self.fetch(bus);
                self.sbc(v);
            }
            0xE5 => {
                let a = self.zp(bus);
                let v = self.rd(bus, a);
                self.sbc(v);
            }
            0xF5 => {
                let a = self.zpx(bus);
                let v = self.rd(bus, a);
                self.sbc(v);
            }
            0xED => {
                let a = self.abs(bus);
                let v = self.rd(bus, a);
                self.sbc(v);
            }
            0xFD => {
                let a = self.abx(bus, true);
                let v = self.rd(bus, a);
                self.sbc(v);
            }
            0xF9 => {
                let a = self.aby(bus, true);
                let v = self.rd(bus, a);
                self.sbc(v);
            }
            0xE1 => {
                let a = self.izx(bus);
                let v = self.rd(bus, a);
                self.sbc(v);
            }
            0xF1 => {
                let a = self.izy(bus, true);
                let v = self.rd(bus, a);
                self.sbc(v);
            }

            0xC9 => {
                let v = self.fetch(bus);
                self.compare(self.a, v);
            }
            0xC5 => {
                let a = self.zp(bus);
                let v = self.rd(bus, a);
                self.compare(self.a, v);
            }
            0xD5 => {
                let a = self.zpx(bus);
                let v = self.rd(bus, a);
                self.compare(self.a, v);
            }
            0xCD => {
                let a = self.abs(bus);
                let v = self.rd(bus, a);
                self.compare(self.a, v);
            }
            0xDD => {
                let a = self.abx(bus, true);
                let v = self.rd(bus, a);
                self.compare(self.a, v);
            }
            0xD9 => {
                let a = self.aby(bus, true);
                let v = self.rd(bus, a);
                self.compare(self.a, v);
            }
            0xC1 => {
                let a = self.izx(bus);
                let v = self.rd(bus, a);
                self.compare(self.a, v);
            }
            0xD1 => {
                let a = self.izy(bus, true);
                let v = self.rd(bus, a);
                self.compare(self.a, v);
            }

            0xE0 => {
                let v = self.fetch(bus);
                self.compare(self.x, v);
            }
            0xE4 => {
                let a = self.zp(bus);
                let v = self.rd(bus, a);
                self.compare(self.x, v);
            }
            0xEC => {
                let a = self.abs(bus);
                let v = self.rd(bus, a);
                self.compare(self.x, v);
            }
            0xC0 => {
                let v = self.fetch(bus);
                self.compare(self.y, v);
            }
            0xC4 => {
                let a = self.zp(bus);
                let v = self.rd(bus, a);
                self.compare(self.y, v);
            }
            0xCC => {
                let a = self.abs(bus);
                let v = self.rd(bus, a);
                self.compare(self.y, v);
            }

            0x24 => {
                let a = self.zp(bus);
                let v = self.rd(bus, a);
                self.bit(v);
            }
            0x2C => {
                let a = self.abs(bus);
                let v = self.rd(bus, a);
                self.bit(v);
            }

            // ---- inc / dec ----
            0xE6 => {
                let a = self.zp(bus);
                self.rmw(bus, a, |c, v| {
                    let r = v.wrapping_add(1);
                    c.set_zn(r);
                    r
                });
            }
            0xF6 => {
                let a = self.zpx(bus);
                self.rmw(bus, a, |c, v| {
                    let r = v.wrapping_add(1);
                    c.set_zn(r);
                    r
                });
            }
            0xEE => {
                let a = self.abs(bus);
                self.rmw(bus, a, |c, v| {
                    let r = v.wrapping_add(1);
                    c.set_zn(r);
                    r
                });
            }
            0xFE => {
                let a = self.abx(bus, false);
                self.rmw(bus, a, |c, v| {
                    let r = v.wrapping_add(1);
                    c.set_zn(r);
                    r
                });
            }
            0xC6 => {
                let a = self.zp(bus);
                self.rmw(bus, a, |c, v| {
                    let r = v.wrapping_sub(1);
                    c.set_zn(r);
                    r
                });
            }
            0xD6 => {
                let a = self.zpx(bus);
                self.rmw(bus, a, |c, v| {
                    let r = v.wrapping_sub(1);
                    c.set_zn(r);
                    r
                });
            }
            0xCE => {
                let a = self.abs(bus);
                self.rmw(bus, a, |c, v| {
                    let r = v.wrapping_sub(1);
                    c.set_zn(r);
                    r
                });
            }
            0xDE => {
                let a = self.abx(bus, false);
                self.rmw(bus, a, |c, v| {
                    let r = v.wrapping_sub(1);
                    c.set_zn(r);
                    r
                });
            }
            0xE8 => {
                self.io(bus);
                self.x = self.x.wrapping_add(1);
                let x = self.x;
                self.set_zn(x);
            }
            0xCA => {
                self.io(bus);
                self.x = self.x.wrapping_sub(1);
                let x = self.x;
                self.set_zn(x);
            }
            0xC8 => {
                self.io(bus);
                self.y = self.y.wrapping_add(1);
                let y = self.y;
                self.set_zn(y);
            }
            0x88 => {
                self.io(bus);
                self.y = self.y.wrapping_sub(1);
                let y = self.y;
                self.set_zn(y);
            }

            // ---- shifts ----
            0x0A => {
                self.io(bus);
                let r = self.asl_val(self.a);
                self.a = r;
            }
            0x06 => {
                let a = self.zp(bus);
                self.rmw(bus, a, |c, v| c.asl_val(v));
            }
            0x16 => {
                let a = self.zpx(bus);
                self.rmw(bus, a, |c, v| c.asl_val(v));
            }
            0x0E => {
                let a = self.abs(bus);
                self.rmw(bus, a, |c, v| c.asl_val(v));
            }
            0x1E => {
                let a = self.abx(bus, false);
                self.rmw(bus, a, |c, v| c.asl_val(v));
            }
            0x4A => {
                self.io(bus);
                let r = self.lsr_val(self.a);
                self.a = r;
            }
            0x46 => {
                let a = self.zp(bus);
                self.rmw(bus, a, |c, v| c.lsr_val(v));
            }
            0x56 => {
                let a = self.zpx(bus);
                self.rmw(bus, a, |c, v| c.lsr_val(v));
            }
            0x4E => {
                let a = self.abs(bus);
                self.rmw(bus, a, |c, v| c.lsr_val(v));
            }
            0x5E => {
                let a = self.abx(bus, false);
                self.rmw(bus, a, |c, v| c.lsr_val(v));
            }
            0x2A => {
                self.io(bus);
                let r = self.rol_val(self.a);
                self.a = r;
            }
            0x26 => {
                let a = self.zp(bus);
                self.rmw(bus, a, |c, v| c.rol_val(v));
            }
            0x36 => {
                let a = self.zpx(bus);
                self.rmw(bus, a, |c, v| c.rol_val(v));
            }
            0x2E => {
                let a = self.abs(bus);
                self.rmw(bus, a, |c, v| c.rol_val(v));
            }
            0x3E => {
                let a = self.abx(bus, false);
                self.rmw(bus, a, |c, v| c.rol_val(v));
            }
            0x6A => {
                self.io(bus);
                let r = self.ror_val(self.a);
                self.a = r;
            }
            0x66 => {
                let a = self.zp(bus);
                self.rmw(bus, a, |c, v| c.ror_val(v));
            }
            0x76 => {
                let a = self.zpx(bus);
                self.rmw(bus, a, |c, v| c.ror_val(v));
            }
            0x6E => {
                let a = self.abs(bus);
                self.rmw(bus, a, |c, v| c.ror_val(v));
            }
            0x7E => {
                let a = self.abx(bus, false);
                self.rmw(bus, a, |c, v| c.ror_val(v));
            }

            // ---- jumps / calls ----
            0x4C => {
                self.pc = self.fetch16(bus);
            }
            0x6C => {
                let ptr = self.fetch16(bus);
                let lo = self.rd(bus, ptr) as u16;
                // 6502 page-wrap bug on the high byte.
                let hi_addr = (ptr & 0xFF00) | (ptr.wrapping_add(1) & 0x00FF);
                let hi = self.rd(bus, hi_addr) as u16;
                self.pc = (hi << 8) | lo;
            }
            0x20 => {
                let lo = self.fetch(bus) as u16;
                self.io(bus);
                self.push16(bus, self.pc);
                let hi = self.rd(bus, self.pc) as u16;
                self.pc = (hi << 8) | lo;
            }
            0x60 => {
                self.io(bus);
                let addr = self.pull16(bus);
                self.io(bus);
                self.pc = addr.wrapping_add(1);
                self.io(bus);
            }
            0x40 => {
                self.io(bus);
                let status = self.pull(bus);
                self.p = (status & !B) | U;
                self.pc = self.pull16(bus);
                self.io(bus);
            }
            0x00 => self.brk(bus),

            // ---- branches ----
            0x10 => {
                let c = self.p & N == 0;
                self.branch(bus, c);
            }
            0x30 => {
                let c = self.p & N != 0;
                self.branch(bus, c);
            }
            0x50 => {
                let c = self.p & V == 0;
                self.branch(bus, c);
            }
            0x70 => {
                let c = self.p & V != 0;
                self.branch(bus, c);
            }
            0x90 => {
                let c = self.p & C == 0;
                self.branch(bus, c);
            }
            0xB0 => {
                let c = self.p & C != 0;
                self.branch(bus, c);
            }
            0xD0 => {
                let c = self.p & Z == 0;
                self.branch(bus, c);
            }
            0xF0 => {
                let c = self.p & Z != 0;
                self.branch(bus, c);
            }

            // ---- flags ----
            0x18 => {
                self.io(bus);
                self.set_flag(C, false);
            }
            0x38 => {
                self.io(bus);
                self.set_flag(C, true);
            }
            0x58 => {
                self.io(bus);
                self.set_flag(I, false);
            }
            0x78 => {
                self.io(bus);
                self.set_flag(I, true);
            }
            0xB8 => {
                self.io(bus);
                self.set_flag(V, false);
            }
            0xD8 => {
                self.io(bus);
                self.set_flag(D, false);
            }
            0xF8 => {
                self.io(bus);
                self.set_flag(D, true);
            }

            // ---- NOPs (official + common unofficial) ----
            0xEA => self.io(bus),
            0x1A | 0x3A | 0x5A | 0x7A | 0xDA | 0xFA => self.io(bus),
            0x80 | 0x82 | 0x89 | 0xC2 | 0xE2 => {
                self.fetch(bus);
            }
            0x04 | 0x44 | 0x64 => {
                let a = self.zp(bus);
                self.rd(bus, a);
            }
            0x14 | 0x34 | 0x54 | 0x74 | 0xD4 | 0xF4 => {
                let a = self.zpx(bus);
                self.rd(bus, a);
            }
            0x0C => {
                let a = self.abs(bus);
                self.rd(bus, a);
            }
            0x1C | 0x3C | 0x5C | 0x7C | 0xDC | 0xFC => {
                let a = self.abx(bus, true);
                self.rd(bus, a);
            }

            // ---- unofficial: LAX / SAX ----
            0xA7 => {
                let a = self.zp(bus);
                let v = self.rd(bus, a);
                self.lda(v);
                self.x = v;
            }
            0xB7 => {
                let a = self.zpy(bus);
                let v = self.rd(bus, a);
                self.lda(v);
                self.x = v;
            }
            0xAF => {
                let a = self.abs(bus);
                let v = self.rd(bus, a);
                self.lda(v);
                self.x = v;
            }
            0xBF => {
                let a = self.aby(bus, true);
                let v = self.rd(bus, a);
                self.lda(v);
                self.x = v;
            }
            0xA3 => {
                let a = self.izx(bus);
                let v = self.rd(bus, a);
                self.lda(v);
                self.x = v;
            }
            0xB3 => {
                let a = self.izy(bus, true);
                let v = self.rd(bus, a);
                self.lda(v);
                self.x = v;
            }
            0x87 => {
                let a = self.zp(bus);
                self.wr(bus, a, self.a & self.x);
            }
            0x97 => {
                let a = self.zpy(bus);
                self.wr(bus, a, self.a & self.x);
            }
            0x8F => {
                let a = self.abs(bus);
                self.wr(bus, a, self.a & self.x);
            }
            0x83 => {
                let a = self.izx(bus);
                self.wr(bus, a, self.a & self.x);
            }

            // ---- unofficial immediate ALU ----
            0x0B | 0x2B => {
                let v = self.fetch(bus);
                self.and(v);
                self.set_flag(C, self.a & 0x80 != 0);
            }
            0x4B => {
                let v = self.fetch(bus);
                self.and(v);
                let a = self.a;
                self.a = self.lsr_val(a);
            }
            0x6B => {
                let v = self.fetch(bus);
                self.and(v);
                let carry = (self.p & C) << 7;
                self.a = (self.a >> 1) | carry;
                self.set_zn(self.a);
                self.set_flag(C, self.a & 0x40 != 0);
                self.set_flag(V, ((self.a >> 5) ^ (self.a >> 6)) & 1 != 0);
            }
            0x8B => {
                let v = self.fetch(bus);
                self.a = self.x & v;
                self.set_zn(self.a);
            }
            0xAB => {
                let v = self.fetch(bus);
                self.a = self.x & v;
                self.x = self.a;
                self.set_zn(self.a);
            }
            0xCB => {
                let v = self.fetch(bus);
                let ax = self.a & self.x;
                let r = ax.wrapping_sub(v);
                self.set_flag(C, ax >= v);
                self.x = r;
                self.set_zn(self.x);
            }

            // ---- unofficial unstable stores / loads ----
            0x93 => {
                let a = self.izy(bus, false);
                let v = self.a & self.x & (((a >> 8) as u8).wrapping_add(1));
                self.wr(bus, a, v);
            }
            0x9B => {
                let a = self.aby(bus, false);
                self.sp = self.a & self.x;
                let v = self.sp & (((a >> 8) as u8).wrapping_add(1));
                self.wr(bus, a, v);
            }
            0x9C => {
                let a = self.abx(bus, false);
                let v = self.y & (((a >> 8) as u8).wrapping_add(1));
                self.wr(bus, a, v);
            }
            0x9E => {
                let a = self.aby(bus, false);
                let v = self.x & (((a >> 8) as u8).wrapping_add(1));
                self.wr(bus, a, v);
            }
            0x9F => {
                let a = self.aby(bus, false);
                let v = self.a & self.x & (((a >> 8) as u8).wrapping_add(1));
                self.wr(bus, a, v);
            }
            0xBB => {
                let a = self.aby(bus, true);
                let v = self.rd(bus, a) & self.sp;
                self.a = v;
                self.x = v;
                self.sp = v;
                self.set_zn(v);
            }

            // ---- unofficial RMW: DCP / ISC / SLO / RLA / SRE / RRA ----
            0xC7 | 0xD7 | 0xCF | 0xDF | 0xDB | 0xC3 | 0xD3 => {
                let a = self.unofficial_addr(bus, opcode);
                self.rmw(bus, a, |c, v| {
                    let r = v.wrapping_sub(1);
                    c.set_flag(C, c.a >= r);
                    c.set_zn(c.a.wrapping_sub(r));
                    r
                });
            }
            0xE7 | 0xF7 | 0xEF | 0xFF | 0xFB | 0xE3 | 0xF3 => {
                let a = self.unofficial_addr(bus, opcode);
                self.rmw(bus, a, |c, v| {
                    let r = v.wrapping_add(1);
                    c.sbc(r);
                    r
                });
            }
            0x07 | 0x17 | 0x0F | 0x1F | 0x1B | 0x03 | 0x13 => {
                let a = self.unofficial_addr(bus, opcode);
                self.rmw(bus, a, |c, v| {
                    let r = c.asl_val(v);
                    c.ora(r);
                    r
                });
            }
            0x27 | 0x37 | 0x2F | 0x3F | 0x3B | 0x23 | 0x33 => {
                let a = self.unofficial_addr(bus, opcode);
                self.rmw(bus, a, |c, v| {
                    let r = c.rol_val(v);
                    c.and(r);
                    r
                });
            }
            0x47 | 0x57 | 0x4F | 0x5F | 0x5B | 0x43 | 0x53 => {
                let a = self.unofficial_addr(bus, opcode);
                self.rmw(bus, a, |c, v| {
                    let r = c.lsr_val(v);
                    c.eor(r);
                    r
                });
            }
            0x67 | 0x77 | 0x6F | 0x7F | 0x7B | 0x63 | 0x73 => {
                let a = self.unofficial_addr(bus, opcode);
                self.rmw(bus, a, |c, v| {
                    let r = c.ror_val(v);
                    c.adc(r);
                    r
                });
            }

            // ---- everything else: treat as NOP to stay robust ----
            _ => self.io(bus),
        }
    }

    /// Resolve the addressing mode for unofficial RMW opcodes by low nibble.
    fn unofficial_addr(&mut self, bus: &mut Bus, opcode: u8) -> u16 {
        match opcode & 0x1F {
            0x07 => self.zp(bus),
            0x17 => self.zpx(bus),
            0x0F => self.abs(bus),
            0x1F => self.abx(bus, false),
            0x1B => self.aby(bus, false),
            0x03 => self.izx(bus),
            0x13 => self.izy(bus, false),
            _ => self.zp(bus),
        }
    }
}

#[inline]
fn page_crossed(a: u16, b: u16) -> bool {
    (a & 0xFF00) != (b & 0xFF00)
}
