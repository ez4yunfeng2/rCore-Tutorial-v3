use alloc::{collections::{BTreeMap, VecDeque}, sync::Arc};
use k210_pac::Interrupt;
use k210_soc::plic::{plic_enable, set_thershold, set_priority, current_irq};
use lazy_static::lazy_static;
use crate::{sync::UPSafeCell, task::TaskControlBlock, sbi::sbi_rustsbi_k210_sext, drivers::BLOCK_DEVICE};

lazy_static!{
    pub static ref IRQMANAGER:Arc<UPSafeCell<IrqManager>> = Arc::new(unsafe{UPSafeCell::new(IrqManager::new())});
}

pub struct IrqManager{
    plic_instance:BTreeMap<usize,VecDeque<TaskControlBlock>>
}

impl IrqManager {
    pub fn new() -> Self {
        let plic_instance = BTreeMap::new();
        Self { plic_instance }
    }

    pub fn register_irq(&mut self,source:Interrupt) {
        plic_enable(source);
        set_priority(source, 1);
        self.plic_instance.insert(source as usize, VecDeque::new());
    }
    #[allow(unused)]
    pub fn irq_wait(&self,source:Interrupt) {
        match source {
            Interrupt::DMA0 => {
                BLOCK_DEVICE.irq_wait();
            }
            _ => {

            }
        }
    }
}

pub fn irq_init() {
    sbi_rustsbi_k210_sext();
    set_thershold(0);
    IRQMANAGER.inner.borrow_mut().register_irq(Interrupt::DMA0);
    println!("Interrupt Init Ok");
}

lazy_static::lazy_static!(
    pub static ref TEMP:UPSafeCell<bool> = unsafe{ UPSafeCell::new(true) };
);

#[no_mangle]
pub fn wait_for_irq() {
    println!("wait_for_irq");
    while *TEMP.exclusive_access() {}
    *TEMP.exclusive_access() = true;
}

pub fn handler_ext_interrupt() {
    unsafe {
        let ptr = k210_pac::DMAC::ptr();
        println!("Status {:#x}",(*ptr).channel[0].intstatus.read().bits());
    }
    *TEMP.exclusive_access() = false;
    println!("[irq] {}",current_irq());
}