use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(in crate::mapper) struct A12EdgeFilter {
    a12_prev: bool,
    a12_low_since: u64,
}

impl A12EdgeFilter {
    pub(in crate::mapper) fn new() -> Self {
        Self {
            a12_prev: false,
            a12_low_since: 0,
        }
    }

    pub(in crate::mapper) fn clocked(
        &mut self,
        addr: u16,
        cycle: u64,
        min_low_cycles: u64,
    ) -> bool {
        let a12 = addr & 0x1000 != 0;
        let clocked =
            a12 && !self.a12_prev && cycle.wrapping_sub(self.a12_low_since) >= min_low_cycles;
        if !a12 && self.a12_prev {
            self.a12_low_since = cycle;
        }
        self.a12_prev = a12;
        clocked
    }
}

impl Default for A12EdgeFilter {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(in crate::mapper) struct CpuCycleIrq {
    irq_counter: u16,
    irq_enabled: bool,
    irq_pending: bool,
}

impl CpuCycleIrq {
    pub(in crate::mapper) fn new() -> Self {
        Self {
            irq_counter: 0,
            irq_enabled: false,
            irq_pending: false,
        }
    }

    pub(in crate::mapper) fn set_enabled(&mut self, enabled: bool, reset_counter: bool) {
        self.irq_enabled = enabled;
        self.irq_pending = false;
        if reset_counter {
            self.irq_counter = 0;
        }
    }

    pub(in crate::mapper) fn enable(&mut self) {
        self.irq_enabled = true;
    }

    pub(in crate::mapper) fn disable(&mut self, reset_counter: bool, clear_pending: bool) {
        self.irq_enabled = false;
        if reset_counter {
            self.irq_counter = 0;
        }
        if clear_pending {
            self.irq_pending = false;
        }
    }

    pub(in crate::mapper) fn set_counter_low(&mut self, value: u8, clear_pending: bool) {
        self.irq_counter = (self.irq_counter & 0xFF00) | value as u16;
        if clear_pending {
            self.irq_pending = false;
        }
    }

    pub(in crate::mapper) fn set_counter_high(&mut self, value: u8, clear_pending: bool) {
        self.irq_counter = (self.irq_counter & 0x00FF) | ((value as u16) << 8);
        if clear_pending {
            self.irq_pending = false;
        }
    }

    pub(in crate::mapper) fn clock_up_to(&mut self, limit: u16, disable_on_hit: bool) {
        if !self.irq_enabled {
            return;
        }
        self.irq_counter = self.irq_counter.wrapping_add(1);
        if self.irq_counter >= limit {
            if disable_on_hit {
                self.irq_enabled = false;
            }
            self.irq_pending = true;
        }
    }

    pub(in crate::mapper) fn clock_up_to_zero(&mut self, disable_on_hit: bool) {
        if !self.irq_enabled {
            return;
        }
        self.irq_counter = self.irq_counter.wrapping_add(1);
        if self.irq_counter == 0 {
            if disable_on_hit {
                self.irq_enabled = false;
            }
            self.irq_pending = true;
        }
    }

    pub(in crate::mapper) fn irq(&self) -> bool {
        self.irq_pending
    }

    pub(in crate::mapper) fn clear(&mut self) {
        self.irq_pending = false;
    }
}

impl Default for CpuCycleIrq {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct Mmc3A12Irq {
    irq_latch: u8,
    irq_counter: u8,
    irq_reload: bool,
    irq_enabled: bool,
    irq_pending: bool,
    #[serde(default)]
    irq_suppress_zero_reload: bool,
    #[serde(flatten)]
    a12: A12EdgeFilter,
}

impl Mmc3A12Irq {
    pub(super) fn new() -> Self {
        Self {
            irq_latch: 0,
            irq_counter: 0,
            irq_reload: false,
            irq_enabled: false,
            irq_pending: false,
            irq_suppress_zero_reload: false,
            a12: A12EdgeFilter::new(),
        }
    }

    pub(super) fn write_latch(&mut self, value: u8) {
        self.irq_latch = value;
    }

    pub(super) fn request_reload(&mut self) {
        self.irq_reload = true;
    }

    pub(super) fn disable(&mut self) {
        self.irq_enabled = false;
        self.irq_pending = false;
    }

    pub(super) fn enable(&mut self) {
        self.irq_enabled = true;
    }

    pub(super) fn notify_a12(&mut self, addr: u16, cycle: u64) {
        // MMC3 counts a rising A12 edge only after A12 stayed low for roughly
        // three CPU cycles. `cycle` is a monotonic PPU-dot count.
        if self.a12.clocked(addr, cycle, 9) {
            self.clock_counter();
        }
    }

    pub(super) fn irq(&self) -> bool {
        self.irq_pending
    }

    pub(super) fn clear(&mut self) {
        self.irq_pending = false;
    }

    fn clock_counter(&mut self) {
        let reset_reload = self.irq_reload;
        let natural_zero_reload = self.irq_counter == 0 && !reset_reload;
        let decrement_to_zero_with_zero_latch =
            self.irq_counter == 1 && self.irq_latch == 0 && !reset_reload;

        if self.irq_counter == 0 || reset_reload {
            self.irq_counter = self.irq_latch;
            self.irq_reload = false;
        } else {
            self.irq_counter -= 1;
        }

        // MMC6-family behavior: if the counter naturally reached 0 while the
        // latch was already 0, the following reload-to-0 edge does not re-assert IRQ.
        let zero_reload_suppressed = natural_zero_reload && self.irq_suppress_zero_reload;
        self.irq_suppress_zero_reload = decrement_to_zero_with_zero_latch;

        if self.irq_counter == 0 && self.irq_enabled && !zero_reload_suppressed {
            self.irq_pending = true;
        }
    }
}

impl Default for Mmc3A12Irq {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mmc3_a12_edge_clocks_after_low_filter() {
        let mut irq = Mmc3A12Irq::new();
        irq.write_latch(1);
        irq.request_reload();
        irq.enable();

        irq.notify_a12(0x1000, 1);
        assert!(!irq.irq());

        irq.notify_a12(0x0000, 2);
        irq.notify_a12(0x1000, 10);
        assert!(!irq.irq());

        irq.notify_a12(0x0000, 11);
        irq.notify_a12(0x1000, 20);
        assert!(!irq.irq());

        irq.notify_a12(0x0000, 21);
        irq.notify_a12(0x1000, 30);
        assert!(irq.irq());
    }

    #[test]
    fn mmc3_zero_reload_suppresses_repeated_natural_zero_irq() {
        let mut irq = Mmc3A12Irq::new();
        irq.write_latch(1);
        irq.request_reload();
        irq.enable();

        irq.notify_a12(0x1000, 9);
        assert!(!irq.irq());

        irq.write_latch(0);

        irq.notify_a12(0x0000, 10);
        irq.notify_a12(0x1000, 19);
        assert!(irq.irq());
        irq.clear();

        irq.notify_a12(0x0000, 20);
        irq.notify_a12(0x1000, 29);
        assert!(!irq.irq());
    }

    #[test]
    fn a12_edge_filter_clocks_only_after_minimum_low_time() {
        let mut filter = A12EdgeFilter::new();

        assert!(!filter.clocked(0x1000, 1, 9));
        assert!(!filter.clocked(0x0000, 2, 9));
        assert!(!filter.clocked(0x1000, 10, 9));
        assert!(!filter.clocked(0x0000, 11, 9));
        assert!(filter.clocked(0x1000, 20, 9));
        assert!(!filter.clocked(0x1000, 30, 9));
    }

    #[test]
    fn cpu_cycle_irq_counts_to_limit_and_disables_on_hit() {
        let mut irq = CpuCycleIrq::new();
        irq.set_enabled(true, true);

        for _ in 0..4095 {
            irq.clock_up_to(4096, true);
        }
        assert!(!irq.irq());

        irq.clock_up_to(4096, true);
        assert!(irq.irq());
        irq.clear();

        irq.clock_up_to(4096, true);
        assert!(!irq.irq());
    }

    #[test]
    fn cpu_cycle_irq_can_count_wrapping_to_zero() {
        let mut irq = CpuCycleIrq::new();
        irq.set_counter_low(0xFE, false);
        irq.set_counter_high(0xFF, false);
        irq.enable();

        irq.clock_up_to_zero(true);
        assert!(!irq.irq());

        irq.clock_up_to_zero(true);
        assert!(irq.irq());
        irq.clear();

        irq.clock_up_to_zero(true);
        assert!(!irq.irq());
    }
}
