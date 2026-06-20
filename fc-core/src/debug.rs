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
    /// Optional condition expression (see [`crate::expr`]); the breakpoint only
    /// fires when it evaluates non-zero. `None` = unconditional.
    #[serde(default)]
    pub condition: Option<String>,
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
        self.add_cond(kind, addr, None)
    }

    /// Add a breakpoint with an optional condition expression.
    pub fn add_cond(&mut self, kind: BpKind, addr: u16, condition: Option<String>) -> u32 {
        self.next_id += 1;
        let id = self.next_id;
        self.breakpoints.push(Breakpoint {
            id,
            kind,
            addr,
            enabled: true,
            condition,
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

    /// Cheap address-only gate: is there any enabled exec breakpoint at `pc`?
    /// (Conditions are checked separately by [`exec_break`] only on a match.)
    pub fn exec_bp_at(&self, pc: u16) -> bool {
        self.breakpoints
            .iter()
            .any(|b| b.enabled && b.kind == BpKind::Exec && b.addr == pc)
    }

    /// Should execution actually break at `pc`? True if any enabled exec
    /// breakpoint at `pc` is unconditional or has a condition evaluating
    /// non-zero against `ctx`.
    pub fn exec_break(&self, pc: u16, ctx: &crate::expr::Ctx) -> bool {
        self.breakpoints.iter().any(|b| {
            b.enabled
                && b.kind == BpKind::Exec
                && b.addr == pc
                && b.condition
                    .as_deref()
                    .map_or(true, |c| crate::expr::eval_cond(c, ctx))
        })
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
