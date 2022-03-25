mod context;
mod id;
mod manager;
mod process;
mod processor;
mod switch;
mod task;

use crate::{
    fs::{open_file, OpenFlags},
    sync::SpinMutex,
};
use alloc::{
    collections::VecDeque,
    string::String,
    sync::Arc,
    vec::{self, Vec},
};
use lazy_static::*;
use manager::fetch_task;
use process::ProcessControlBlock;
use switch::__switch;

pub use context::TaskContext;
pub use id::{kstack_alloc, pid_alloc, KernelStack, PidHandle};
pub use manager::add_task;
pub use processor::{
    current_hartid, current_kstack_top, current_process, current_processor, current_task,
    current_trap_cx, current_trap_cx_user_va, current_user_token, enter_kernel, exit_kernel,
    init_hart, run_tasks, schedule, take_current_task,
};
pub use task::{TaskControlBlock, TaskStatus, Tms};

pub fn suspend_current_and_run_next() {
    // There must be an application running.
    let task = match take_current_task() {
        Some(task) => task,
        None => {
            println!("No Task");
            return;
        }
    };
    // ---- access current TCB exclusively
    let mut task_inner = task.inner_lock_access();
    let task_cx_ptr = &mut task_inner.task_cx as *mut TaskContext;
    // Change status to Ready
    task_inner.task_status = TaskStatus::Ready;
    drop(task_inner);
    // ---- release current TCB

    // push back to ready queue.
    add_task(task);

    // jump to scheduling cycle
    schedule(task_cx_ptr);
}

pub fn block_current_and_run_next() {
    let task = take_current_task().unwrap();
    let mut task_inner = task.inner_lock_access();
    let task_cx_ptr = &mut task_inner.task_cx as *mut TaskContext;
    task_inner.task_status = TaskStatus::Blocking;
    drop(task_inner);
    schedule(task_cx_ptr);
}

pub fn exit_current_and_run_next(exit_code: i32) {
    
    let task = take_current_task().unwrap();
    let mut task_inner = task.inner_lock_access();
    let process = task.process.upgrade().unwrap();

    let tid = task_inner.res.as_ref().unwrap().tid;
    // record exit code
    task_inner.exit_code = Some(exit_code);
    task_inner.res = None;
    // here we do not remove the thread since we are still using the kstack
    // it will be deallocated when sys_waittid is called
    drop(task_inner);
    drop(task);
    // however, if this is the main thread of current process
    // the process should terminate at once
    if tid == 0 {
        
        let mut process_inner = process.try_inner_exclusive_access().unwrap();
        // mark this process as a zombie process
        process_inner.is_zombie = true;
        // record exit code of main process
        process_inner.exit_code = exit_code;
        
        {
            // move all child processes under init process
            let mut initproc_inner = INITPROC.try_inner_exclusive_access().unwrap();
            for child in process_inner.children.iter() {
                child.try_inner_exclusive_access().unwrap().parent = Some(Arc::downgrade(&INITPROC));
                initproc_inner.children.push(child.clone());
            }
        }

        // deallocate user res (including tid/trap_cx/ustack) of all threads
        // it has to be done before we dealloc the whole memory_set
        // otherwise they will be deallocated twice
        for task in process_inner.tasks.iter().filter(|t| t.is_some()) {
            let task = task.as_ref().unwrap();
            let mut task_inner = task.inner_lock_access();
            task_inner.res = None;
        }

        process_inner.children.clear();
        // deallocate other data in user space i.e. program code/data section
        process_inner.memory_set.recycle_data_pages();
    }
    drop(process);
    // we do not have to save task context
    let mut _unused = TaskContext::zero_init();
    println!("exit");
    schedule(&mut _unused as *mut _);
}

lazy_static! {
    pub static ref INITPROC: Arc<ProcessControlBlock> = {
        let v = include_bytes!("../usertests");
        // let inode = open_file("usertests", OpenFlags::RDONLY).unwrap();
        // let v = inode.read_all();
        ProcessControlBlock::new(v.as_slice())
    };
}

pub fn add_initproc() {
    let v = include_bytes!("../usertests");
    let _initproc = INITPROC.clone();
}