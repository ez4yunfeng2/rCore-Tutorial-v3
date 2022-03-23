use super::manager::TASK_MANAGER;
use super::{__switch, manager};
use super::{fetch_task, TaskStatus};
use super::{ProcessControlBlock, TaskContext, TaskControlBlock};
use crate::config::MAX_HARTID;
use crate::sync::{intr_on, intr_off};
use crate::trap::TrapContext;
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use riscv::register::sstatus;

pub static mut PROCESSORS: BTreeMap<usize, Processor> = BTreeMap::new();

pub struct Processor {
    pub noff: usize,
    pub intena: bool,
    current: Option<Arc<TaskControlBlock>>,
    idle_task_cx: TaskContext,
}

pub fn new() -> Processor {
    Processor {
        noff: 0,
        intena: false,
        current: None,
        idle_task_cx: TaskContext::zero_init(),
    }
}

impl Processor {
    pub fn new() -> Self {
        Self {
            noff: 0,
            intena: false,
            current: None,
            idle_task_cx: TaskContext::zero_init(),
        }
    }
    fn get_idle_task_cx_ptr(&mut self) -> *mut TaskContext {
        &mut self.idle_task_cx as *mut _
    }
    pub fn take_current(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.current.take()
    }
    pub fn current(&self) -> Option<Arc<TaskControlBlock>> {
        self.current.as_ref().map(|task| Arc::clone(task))
    }
}

pub fn current_hartid() -> usize {
    let mut tp: usize;
    unsafe { asm!("mv {}, tp", out(reg) tp) };
    tp
}

pub fn init_hart() {
    unsafe {
        for hartid in 0..MAX_HARTID {
            PROCESSORS.insert(hartid, Processor::new());
        }
    }
}

pub fn run_tasks(hartid: usize) {
    loop {
        let mut processor = current_processor().unwrap();
        intr_on();
        if let Some(task) = fetch_task() {
            intr_off();
            let mut task_inner = task.inner_lock_access();
            let idle_task_cx_ptr = processor.get_idle_task_cx_ptr();
            let next_task_cx_ptr = &task_inner.task_cx as *const TaskContext;
            task_inner.task_status = TaskStatus::Running;
            drop(task_inner);
            processor.current = Some(task);
            assert_eq!(sstatus::read().sie(), false);
            unsafe {
                __switch(idle_task_cx_ptr, next_task_cx_ptr);
            }
        }
    }
}

pub fn current_processor() -> Option<&'static mut Processor> {
    unsafe { PROCESSORS.get_mut(&current_hartid()) }
}

pub fn take_current_task() -> Option<Arc<TaskControlBlock>> {
    let processor = current_processor().unwrap();
    processor.take_current()
}

pub fn current_task() -> Option<Arc<TaskControlBlock>> {
    let processor = current_processor().unwrap();
    processor.current()
}

pub fn current_process() -> Arc<ProcessControlBlock> {
    current_task()
        .unwrap()
        .process
        .upgrade()
        .unwrap()
}

pub fn current_user_token() -> usize {
    let task = current_task().unwrap();
    let token = task.get_user_token();
    token
}

pub fn current_trap_cx() -> &'static mut TrapContext {
    current_task().unwrap().inner_lock_access().get_trap_cx()
}

pub fn current_trap_cx_user_va() -> usize {
    current_task()
        .unwrap()
        .inner_lock_access()
        .res
        .as_ref()
        .unwrap()
        .trap_cx_user_va()
}

pub fn current_kstack_top() -> usize {
    current_task()
        .unwrap()
        .kstack
        .get_top()
}

pub fn schedule(switched_task_cx_ptr: *mut TaskContext) {
    intr_off();
    let processor = current_processor().unwrap();
    let idle_task_cx_ptr = processor.get_idle_task_cx_ptr();
    drop(processor);
    unsafe {
        __switch(switched_task_cx_ptr, idle_task_cx_ptr);
    }
}
