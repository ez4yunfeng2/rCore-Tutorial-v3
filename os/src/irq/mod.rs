use crate::{
    drivers::BLOCK_DEVICE, sbi::sbi_rustsbi_k210_sext, sync::UPSafeCell, task::TaskControlBlock,
};
use alloc::{
    collections::{BTreeMap, VecDeque},
    sync::Arc,
};
use k210_pac::Interrupt;
use k210_soc::{
    dmac::channel_interrupt_clear,
    plic::{plic_enable, set_priority, set_thershold},
    sysctl::dma_channel,
};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref TEMP: UPSafeCell<bool> = unsafe { UPSafeCell::new(true) };
    pub static ref IRQMANAGER: Arc<UPSafeCell<IrqManager>> =
        Arc::new(unsafe { UPSafeCell::new(IrqManager::new()) });
}

pub struct IrqManager {
    plic_instance: BTreeMap<usize, VecDeque<TaskControlBlock>>,
}

impl IrqManager {
    pub fn new() -> Self {
        let plic_instance = BTreeMap::new();
        Self { plic_instance }
    }

    pub fn register_irq(&mut self, source: Interrupt) {
        plic_enable(source);
        set_priority(source, 1);
        self.plic_instance.insert(source as usize, VecDeque::new());
    }
    #[allow(unused)]
    pub fn irq_wait(&self, source: Interrupt) {
        match source {
            Interrupt::DMA0 => {
                BLOCK_DEVICE.irq_wait();
            }
            _ => {}
        }
    }
}

pub fn irq_init() {
    sbi_rustsbi_k210_sext();
    set_thershold(0);
    IRQMANAGER.exclusive_access().register_irq(Interrupt::DMA0);
    println!("Interrupt Init Ok");
}

#[no_mangle]
pub fn wait_for_irq() {
    while *TEMP.exclusive_access() {}
    *TEMP.exclusive_access() = true;
}

pub unsafe fn handler_ext() {
    let ptr = k210_pac::PLIC::ptr();
    let irq = (*ptr).targets[0].claim.read().bits();
    match irq {
        27 => {
            channel_interrupt_clear(dma_channel::CHANNEL0);
            *TEMP.exclusive_access() = false;
        }
        33 => {}
        _ => {
            panic!("unknow irq")
        }
    }
    (*ptr).targets[0].claim.write(|w| w.bits(irq));
}
