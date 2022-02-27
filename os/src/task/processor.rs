use super::__switch;
use super::{fetch_task, TaskStatus};
use super::{ProcessControlBlock, TaskContext, TaskControlBlock};
use crate::trap::TrapContext;
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use k210_soc::sleep::usleep;
use lazy_static::*;
use spin::mutex::Mutex;

pub struct Processor {
    current: Option<Arc<TaskControlBlock>>,
    idle_task_cx: TaskContext,
}

impl Processor {
    pub fn new() -> Self {
        Self {
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

lazy_static! {
    pub static ref PROCESSORS: Mutex<BTreeMap<usize,Processor>> = Mutex::new( BTreeMap::new()) ;
}

pub fn current_hartid() -> usize {
    let mut tp:usize;
    unsafe { asm!("mv {}, tp", out(reg) tp) };
    tp
}

pub fn run_tasks(hartid : usize) {
    PROCESSORS.lock().insert(hartid, Processor::new() );
    loop {
        usleep(1000000);
        // println!("hartid {} {}",hartid, current_hartid());
        let mut processors = PROCESSORS.lock();
        let mut processor = processors.get_mut(&hartid).unwrap();
        if let Some(task) = fetch_task() {
            println!("[GetTask]: {}",hartid);
            let idle_task_cx_ptr = processor.get_idle_task_cx_ptr();
            // access coming task TCB exclusively
            let mut task_inner = task.inner_exclusive_access();
            let next_task_cx_ptr = &task_inner.task_cx as *const TaskContext;
            task_inner.task_status = TaskStatus::Running;
            drop(task_inner);
            // release coming task TCB manually
            processor.current = Some(task);
            // release processor manually
            drop(processors);
            // println!("]  hartid: {} {:#x} {:#x}",hartid,idle_task_cx_ptr as usize,next_task_cx_ptr as usize);
            unsafe {
                __switch(idle_task_cx_ptr, next_task_cx_ptr);
            }
        } else {
            println!("no tasks available in run_tasks hartid {}", hartid);
        }
    }
}

pub fn take_current_task() -> Option<Arc<TaskControlBlock>> {
    let mut core_manager = PROCESSORS.lock();
    let processor = core_manager.get_mut(&current_hartid()).unwrap();
    processor.take_current()
}

pub fn current_task() -> Option<Arc<TaskControlBlock>> {
    let core_manager = PROCESSORS.lock();
    let processor = core_manager.get(&current_hartid()).unwrap();
    processor.current()
}

pub fn current_process() -> Arc<ProcessControlBlock> {
    current_task().unwrap().process.upgrade().unwrap()
}

pub fn current_user_token() -> usize {
    let task = current_task().unwrap();
    let token = task.get_user_token();
    token
}

pub fn current_trap_cx() -> &'static mut TrapContext {
    current_task()
        .unwrap()
        .inner_exclusive_access()
        .get_trap_cx()
}

pub fn current_trap_cx_user_va() -> usize {
    current_task()
        .unwrap()
        .inner_exclusive_access()
        .res
        .as_ref()
        .unwrap()
        .trap_cx_user_va()
}

// pub fn current_kstack_top() -> usize {
//     match current_task() {
//         Some(task) => task.kstack.get_top(),
//         None => 0,
//     }
// }

pub fn schedule(switched_task_cx_ptr: *mut TaskContext) {
    let mut core_manager = PROCESSORS.lock();
    let processor = core_manager.get_mut(&current_hartid()).unwrap();
    let idle_task_cx_ptr = processor.get_idle_task_cx_ptr();
    drop(core_manager);
    unsafe {
        __switch(switched_task_cx_ptr, idle_task_cx_ptr);
    }
}
