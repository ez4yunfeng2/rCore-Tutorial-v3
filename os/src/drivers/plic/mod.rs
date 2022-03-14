use alloc::sync::Arc;
use k210_pac::{Peripherals, PLIC, Interrupt};
use k210_soc::plic::{current_irq, clear_irq};

use crate::sync::UPSafeCell;

use super::PlicDevice;

lazy_static::lazy_static!(
    pub static ref PLIC_DRIVE: Arc<dyn PlicDevice> = Arc::new(Plic::new());
);

struct Plic(UPSafeCell<PLIC>);

impl Plic {
    pub fn new() -> Plic {
        Plic(unsafe{ UPSafeCell::new(Peripherals::steal().PLIC)})
    }
}

impl PlicDevice for Plic {
    
    fn current(&self, hartid: usize) -> usize {
        self.0.exclusive_access().targets[hartid].claim.read().bits() as usize
    }

    fn clear(&self, irq: usize, hartid: usize) {
        self.0.exclusive_access().targets[hartid].claim.write(|w|unsafe{ w.bits(irq as u32) });
    }

    fn set_thershold(&self,value: u32, hartid: usize) {
        self.0.exclusive_access().targets[hartid].threshold.write(|w| unsafe{ w.bits(value) });
    }
    fn set_priority(&self, value: u32, pin: Interrupt) {
        self.0.exclusive_access().priority[pin as usize].write(|w| unsafe{ w.bits(value) })
    }

    fn enable(&self, source: Interrupt, hartid: usize) {
        let idx = source as usize;
        self.0.exclusive_access().target_enables[hartid].enable[idx / 32]
        .modify(|r, w|unsafe{ w.bits(set_bit(r.bits(), idx as u8 % 32, true)) });
    }

}
#[inline]
pub fn set_bit(inval: u32, bit: u8, state: bool) -> u32 {
    if state {
        inval | (1 << u32::from(bit))
    } else {
        inval & !(1 << u32::from(bit))
    }
}