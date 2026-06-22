use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct Mmc3A12Irq {
    irq_latch: u8,
    irq_counter: u8,
    irq_reload: bool,
    irq_enabled: bool,
    irq_pending: bool,
    #[serde(default)]
    irq_suppress_zero_reload: bool,
    a12_prev: bool,
    a12_low_since: u64,
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
            a12_prev: false,
            a12_low_since: 0,
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
        let a12 = addr & 0x1000 != 0;
        if a12 && !self.a12_prev {
            // MMC3 counts a rising A12 edge only after A12 stayed low for
            // roughly three CPU cycles. `cycle` is a monotonic PPU-dot count.
            if cycle.wrapping_sub(self.a12_low_since) >= 9 {
                self.clock_counter();
            }
        } else if !a12 && self.a12_prev {
            self.a12_low_since = cycle;
        }
        self.a12_prev = a12;
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
}
