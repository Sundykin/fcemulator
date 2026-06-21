//! Access heatmap (roadmap L4.4) — per-address read / write / exec counters and a
//! per-frame-decaying "recently hot" map over the 64 KB CPU address space, with a
//! code/data classification (Mesen `CodeDataLogger` intent) derived from the
//! counts.
//!
//! Pure side-channel and **off by default**: the heatmap is `None` until enabled,
//! so it costs nothing (no allocation, no taps) when off. When on, the Bus taps
//! `read`/`write` and the CPU taps opcode `fetch` (= exec), all behind the shared
//! `Bus::observing` gate. Transient — never serialized.
//!
//! Code/data is derived, not flagged: every opcode/operand byte is both read once
//! (via `bus.read`) and exec'd once (via the fetch tap), so `read == exec` marks a
//! pure-code byte, while `read > exec` means it was *also* read as data. Thus
//! `code = exec > 0` and `data = write > 0 || read > exec`.

const SPACE: usize = 0x1_0000;

/// One hot address in a heatmap summary.
#[derive(Debug, Clone, Copy)]
pub struct HotAddr {
    pub addr: u16,
    pub read: u32,
    pub write: u32,
    pub exec: u32,
    pub code: bool,
    pub data: bool,
    pub recency: u16,
}

/// Dense per-CPU-address access counters. Boxed slices are heap-allocated
/// directly (no 256 KB stack temporaries).
#[derive(Debug, Clone)]
pub struct Heatmap {
    read: Box<[u32]>,
    write: Box<[u32]>,
    exec: Box<[u32]>,
    recency: Box<[u16]>,
}

impl Default for Heatmap {
    fn default() -> Self {
        Self::new()
    }
}

impl Heatmap {
    pub fn new() -> Self {
        Heatmap {
            read: vec![0u32; SPACE].into_boxed_slice(),
            write: vec![0u32; SPACE].into_boxed_slice(),
            exec: vec![0u32; SPACE].into_boxed_slice(),
            recency: vec![0u16; SPACE].into_boxed_slice(),
        }
    }

    #[inline]
    pub fn tap_read(&mut self, addr: u16) {
        let i = addr as usize;
        self.read[i] = self.read[i].saturating_add(1);
        self.recency[i] = u16::MAX;
    }
    #[inline]
    pub fn tap_write(&mut self, addr: u16) {
        let i = addr as usize;
        self.write[i] = self.write[i].saturating_add(1);
        self.recency[i] = u16::MAX;
    }
    #[inline]
    pub fn tap_exec(&mut self, addr: u16) {
        let i = addr as usize;
        self.exec[i] = self.exec[i].saturating_add(1);
        self.recency[i] = u16::MAX;
    }

    /// Per-frame multiplicative decay (×7/8) of the "recently hot" map; the raw
    /// read/write/exec totals are cumulative and untouched.
    pub fn decay(&mut self) {
        for r in self.recency.iter_mut() {
            *r -= *r >> 3;
        }
    }

    pub fn reset(&mut self) {
        self.read.iter_mut().for_each(|v| *v = 0);
        self.write.iter_mut().for_each(|v| *v = 0);
        self.exec.iter_mut().for_each(|v| *v = 0);
        self.recency.iter_mut().for_each(|v| *v = 0);
    }

    #[inline]
    fn total_at(&self, i: usize) -> u64 {
        self.read[i] as u64 + self.write[i] as u64 + self.exec[i] as u64
    }

    fn hot_at(&self, i: usize) -> HotAddr {
        let (r, w, x) = (self.read[i], self.write[i], self.exec[i]);
        HotAddr {
            addr: i as u16,
            read: r,
            write: w,
            exec: x,
            code: x > 0,
            data: w > 0 || r > x,
            recency: self.recency[i],
        }
    }

    /// The `n` addresses with the most total accesses (read+write+exec), descending.
    pub fn hottest(&self, n: usize) -> Vec<HotAddr> {
        let mut idx: Vec<usize> = (0..SPACE).filter(|&i| self.total_at(i) > 0).collect();
        idx.sort_unstable_by(|&a, &b| self.total_at(b).cmp(&self.total_at(a)));
        idx.truncate(n);
        idx.into_iter().map(|i| self.hot_at(i)).collect()
    }

    /// Total accesses per 256-byte page (256 entries) — a compact overview for a
    /// memory heatmap strip.
    pub fn page_totals(&self) -> Vec<u64> {
        (0..256)
            .map(|p| {
                let base = p * 256;
                (base..base + 256).map(|i| self.total_at(i)).sum()
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_classify_and_decay() {
        let mut hm = Heatmap::new();
        for _ in 0..5 {
            hm.tap_read(0x2002); // polled register → data
        }
        hm.tap_write(0x0300); // RAM write → data
        hm.tap_exec(0x8000); // single fetch → code (read==exec not reached here)
        // emulate a code byte: read once (bus.read) + exec once (fetch tap)
        hm.tap_read(0x8001);
        hm.tap_exec(0x8001);

        let hot = hm.hottest(16);
        let p2002 = hot.iter().find(|h| h.addr == 0x2002).unwrap();
        assert_eq!(p2002.read, 5);
        assert!(p2002.data && !p2002.code);
        let code = hot.iter().find(|h| h.addr == 0x8001).unwrap();
        assert!(code.code && !code.data, "read==exec → pure code, not data");
        assert!(hot.iter().find(|h| h.addr == 0x0300).unwrap().data);

        let before = hm.page_totals().iter().sum::<u64>();
        hm.decay();
        assert!(hm.hottest(16).iter().find(|h| h.addr == 0x2002).unwrap().recency < u16::MAX);
        assert_eq!(hm.page_totals().iter().sum::<u64>(), before, "decay leaves totals");
        hm.reset();
        assert!(hm.hottest(16).is_empty());
    }
}
