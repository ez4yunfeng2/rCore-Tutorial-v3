use crate::{
    sync::UPSafeCell, task::{TaskControlBlock, take_current_task, TaskContext, schedule, add_task, TaskStatus}, drivers::{BLOCK_DEVICE, UART_DEVICE},
};
use alloc::{
    collections::{BTreeMap, VecDeque},
    sync::Arc,
};
use k210_pac::Interrupt;
use k210_soc::{
    dmac::channel_interrupt_clear,
    plic::{plic_enable, set_priority, set_thershold, current_irq, clear_irq},
    sysctl::dma_channel,
};
use lazy_static::lazy_static;
use riscv::register::{sstatus, sie};

lazy_static! {
    pub static ref FLAG: UPSafeCell<bool> = unsafe { UPSafeCell::new(true) };
    pub static ref IRQMANAGER: Arc<UPSafeCell<IrqManager>> =
        Arc::new(unsafe { UPSafeCell::new(IrqManager::new()) });
}

pub struct IrqManager {
    plic_instance: BTreeMap<usize, VecDeque<Arc<TaskControlBlock>>>,
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
    pub fn inqueue(&mut self, irq: usize, task: Arc<TaskControlBlock>) {
        if let Some(queue) = self.plic_instance.get_mut(&irq) {
            queue.push_back(task)
        }
    }
    pub fn dequeue(&mut self, irq: usize) -> Option<Arc<TaskControlBlock>> {
        if let Some(queue) = self.plic_instance.get_mut(&irq) {
            queue.pop_front()
        } else {
            None
        }
    }

}

pub fn irq_init() {
    unsafe {
        sie::set_ssoft();
        set_thershold(0);
        IRQMANAGER.exclusive_access().register_irq(Interrupt::DMA0);
        IRQMANAGER.exclusive_access().register_irq(Interrupt::UARTHS);
        println!("Interrupt Init Ok");
    }
}
#[no_mangle]
pub fn wait_for_irq_and_run_next(irq: usize) {
    if let Some(task) = take_current_task() {
        
        let mut task_inner = task.inner_lock_access();
        task_inner.task_status = TaskStatus::Waiting;
        let task_cx_ptr = &mut task_inner.task_cx as *mut TaskContext;
        drop(task_inner);
        IRQMANAGER.exclusive_access().inqueue(irq, task);
        schedule(task_cx_ptr);
    } else {
        panic!("Fuck")
    }
}

pub fn handler_ext() {
    let irq = current_irq();
    match irq {
        27 => {
            BLOCK_DEVICE.handler_interrupt();
            let task =  IRQMANAGER.exclusive_access().dequeue(irq).unwrap();
            add_task(task);
        }
        33 => {
            UART_DEVICE.handler_interrupt();
            match IRQMANAGER.exclusive_access().dequeue(irq) {
                Some(task) => add_task(task),
                None => {},
            }
        }
        _ => {
            panic!("unknow irq")
        }
    }
    clear_irq(irq);
}
