//! Debugger state: execute / read / write breakpoints, halt tracking, and an
//! execution-trace ring buffer. Driven by [`ControlDeck`](crate::ControlDeck).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BpKind {
    Exec,
    Read,
    Write,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Breakpoint {
    pub id: u32,
    pub kind: BpKind,
    pub addr: u16,
    pub enabled: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Debugger {
    pub breakpoints: Vec<Breakpoint>,
    /// PC at which execution is currently halted (None = running).
    pub halted: Option<u16>,
    next_id: u32,
    /// Skip the execute-breakpoint check for the next instruction (so resuming
    /// from a breakpoint doesn't immediately re-trigger it).
    pub(crate) skip_once: bool,
}

impl Debugger {
    pub fn add(&mut self, kind: BpKind, addr: u16) -> u32 {
        self.next_id += 1;
        let id = self.next_id;
        self.breakpoints.push(Breakpoint {
            id,
            kind,
            addr,
            enabled: true,
        });
        id
    }

    pub fn remove(&mut self, id: u32) {
        self.breakpoints.retain(|b| b.id != id);
    }

    /// Toggle an execute breakpoint at `addr` (add if absent, remove if present).
    pub fn toggle_exec(&mut self, addr: u16) {
        if let Some(pos) = self
            .breakpoints
            .iter()
            .position(|b| b.kind == BpKind::Exec && b.addr == addr)
        {
            self.breakpoints.remove(pos);
        } else {
            self.add(BpKind::Exec, addr);
        }
    }

    pub fn exec_bp_at(&self, pc: u16) -> bool {
        self.breakpoints
            .iter()
            .any(|b| b.enabled && b.kind == BpKind::Exec && b.addr == pc)
    }

    pub fn addrs(&self, kind: BpKind) -> Vec<u16> {
        self.breakpoints
            .iter()
            .filter(|b| b.enabled && b.kind == kind)
            .map(|b| b.addr)
            .collect()
    }

    pub fn has_any(&self) -> bool {
        self.breakpoints.iter().any(|b| b.enabled)
    }
}
