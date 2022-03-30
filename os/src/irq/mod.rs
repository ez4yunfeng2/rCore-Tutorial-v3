use crate::{
    drivers::{PlicDevice, BLOCK_DEVICE, PLIC_DRIVE, UART_DEVICE},
    sync::{intr_on, SpinMutex, UPSafeCell},
    task::{
        add_task, current_hartid, schedule, take_current_task, TaskContext, TaskControlBlock,
        TaskStatus,
    },
};
use alloc::{
    collections::{BTreeMap, VecDeque},
    sync::Arc,
};
use k210_pac::{dmac::channel::ctl::SINC_R, Interrupt};
use k210_soc::{
    dmac::channel_interrupt_clear,
    plic::{clear_irq, current_irq, plic_enable, set_priority, set_thershold},
    sysctl::dma_channel,
};
use lazy_static::lazy_static;
use riscv::register::{sie, sstatus};

lazy_static! {
    pub static ref IRQMANAGER: Arc<SpinMutex<IrqManager>> =
        Arc::new(SpinMutex::new(IrqManager::new()));
}

pub struct IrqManager {
    plic_instance: Arc<dyn PlicDevice>,
    task_list: BTreeMap<usize, VecDeque<Arc<TaskControlBlock>>>,
}

impl IrqManager {
    pub fn new() -> Self {
        Self {
            plic_instance: PLIC_DRIVE.clone(),
            task_list: BTreeMap::new(),
        }
    }

    pub fn register_irq(&mut self, source: Interrupt, hartid: usize) {
        self.plic_instance.enable(source, hartid);
        self.plic_instance.set_priority(1, source);
        self.task_list.insert(source as usize, VecDeque::new());
    }
    pub fn inqueue(&mut self, irq: usize, task: Arc<TaskControlBlock>) {
        if let Some(queue) = self.task_list.get_mut(&irq) {
            queue.push_back(task)
        }
    }
    pub fn dequeue(&mut self, irq: usize) -> Option<Arc<TaskControlBlock>> {
        if let Some(queue) = self.task_list.get_mut(&irq) {
            queue.pop_front()
        } else {
            None
        }
    }

    pub fn set_thershold(&mut self, value: u32, hartid: usize) {
        self.plic_instance.set_thershold(value, hartid)
    }

    pub fn clear(&self, irq: usize, hartid: usize) {
        self.plic_instance.clear(irq, hartid)
    }
}

pub fn irq_init(hartid: usize) {
    unsafe {
        let mut irq_manager = IRQMANAGER.lock();
        sie::set_ssoft();
        irq_manager.set_thershold(0, hartid);
        irq_manager.register_irq(Interrupt::DMA0, hartid);
        irq_manager.register_irq(Interrupt::UARTHS, hartid);
    }
}

#[no_mangle]
pub fn wait_for_irq_and_run_next(irq: usize) {
    intr_check!();
    let mut irq_manager = IRQMANAGER.lock();
    if let Some(task) = take_current_task() {
        let mut task_inner = task.inner_lock_access();
        task_inner.task_status = TaskStatus::Waiting;
        let task_cx_ptr = &mut task_inner.task_cx as *mut TaskContext;
        drop(task_inner);
        irq_manager.inqueue(irq, task);
        drop(irq_manager);
        schedule(task_cx_ptr);
    } else {
        panic!("too early irq");
    }
}

pub fn handler_ext() {
    intr_check!();
    let mut irq_manager = IRQMANAGER.lock();
    let irq = PLIC_DRIVE.current(current_hartid());
    match irq {
        0 => {}
        27 => {
            BLOCK_DEVICE.handler_interrupt();
            match irq_manager.dequeue(irq) {
                Some(task) => add_task(task),
                None => {}
            }
        }
        33 => {
            UART_DEVICE.handler_interrupt();
            match irq_manager.dequeue(irq) {
                Some(task) => add_task(task),
                None => {}
            }
        }
        _ => {
            panic!("Unsupported irq {}", irq)
        }
    }
    if irq != 0 {
        irq_manager.clear(irq, current_hartid());
    }
}
