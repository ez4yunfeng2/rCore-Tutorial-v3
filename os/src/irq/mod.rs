use alloc::{collections::{BTreeMap, VecDeque}, sync::Arc};
use k210_pac::Interrupt;
use k210_soc::{plic::{plic_enable, clear_irq,set_thershold, set_priority, current_irq}, sysctl::dma_channel, dmac::{channel_interrupt_clear, enable_channel_interrupt}};
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
    sbi_rustsbi_k210_sext(handler_ext_interrupt as usize);
    set_thershold(0);
    enable_channel_interrupt(dma_channel::CHANNEL0);
    IRQMANAGER.exclusive_access().register_irq(Interrupt::DMA0);
    println!("handler_ext_interrupt {:#x}",handler_ext_interrupt as usize);
}

pub fn handler_ext_interrupt() {
    unsafe {
        let ptr = k210_pac::DMAC::ptr();
        ;
        println!("Status {:#x}",(*ptr).channel[0].status.read().bits());
    }
    
    let irq = current_irq();
    println!("Fuck You {}",irq);
    
    channel_interrupt_clear(dma_channel::CHANNEL0);
    // clear_irq();
}

pub fn wait_for_irq(source:Interrupt) {
    IRQMANAGER.exclusive_access().irq_wait(source)
}