use super::id::TaskUserRes;
use super::{kstack_alloc, KernelStack, ProcessControlBlock, TaskContext};
use crate::sync::{SpinMutex, SpinMutexGuard};
use crate::timer::get_time_ms;
use crate::trap::TrapContext;
use crate::{mm::PhysPageNum, sync::UPSafeCell};
use alloc::sync::{Arc, Weak};
use core::ops::{Deref, DerefMut};

pub struct TaskControlBlock {
    // immutable
    pub process: Weak<ProcessControlBlock>,
    pub kstack: KernelStack,
    // mutable
    pub inner: SpinMutex<TaskControlBlockInner>,
}

impl TaskControlBlock {
    pub fn inner_lock_access(&self) -> SpinMutexGuard<'_, TaskControlBlockInner> {
        self.inner.lock()
    }

    pub fn get_user_token(&self) -> usize {
        // let process = self.process.upgrade().unwrap();
        // let inner = process.try_inner_exclusive_access().unwrap();
        // inner.memory_set.token()
        match self.process.upgrade() {
            Some(process) => {
                let inner = process.try_inner_exclusive_access().unwrap();
                return inner.memory_set.token()
            },
            None => panic!("None"),
        }
    }
}

pub struct TaskControlBlockInner {
    pub ticks: usize,
    pub times: Tms,
    pub res: Option<TaskUserRes>,
    pub trap_cx_ppn: PhysPageNum,
    pub task_cx: TaskContext,
    pub task_status: TaskStatus,
    pub exit_code: Option<i32>,
}

impl TaskControlBlockInner {
    pub fn get_trap_cx(&self) -> &'static mut TrapContext {
        self.trap_cx_ppn.get_mut()
    }

    #[allow(unused)]
    fn get_status(&self) -> TaskStatus {
        self.task_status
    }
    pub fn enter_kernel(&mut self) {
        let ticks = get_time_ms();
        self.times.tms_utime += (ticks - self.ticks);
        self.ticks = ticks;
    }
    pub fn exit_kernel(&mut self) {
        let ticks = get_time_ms();
        self.times.tms_stime += (ticks - self.ticks);
        self.ticks = ticks;
    }
}

impl TaskControlBlock {
    pub fn new(
        process: Arc<ProcessControlBlock>,
        ustack_base: usize,
        alloc_user_res: bool,
    ) -> Self {
        let res = TaskUserRes::new(Arc::clone(&process), ustack_base, alloc_user_res);
        let trap_cx_ppn = res.trap_cx_ppn();
        let kstack = kstack_alloc();
        let kstack_top = kstack.get_top();
        Self {
            process: Arc::downgrade(&process),
            kstack,
            inner: SpinMutex::new(TaskControlBlockInner {
                ticks: 0,
                times: Tms::default(),
                res: Some(res),
                trap_cx_ppn,
                task_cx: TaskContext::goto_trap_return(kstack_top),
                task_status: TaskStatus::Ready,
                exit_code: None,
            }),
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    Ready,
    Waiting,
    Running,
    Blocking,
}

#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct Tms {
    tms_utime: usize,
    tms_stime: usize,
    tms_cutime: usize,
    tms_cstime: usize,
}

impl Deref for Tms {
    type Target = Self;

    fn deref(&self) -> &Self::Target {
        self
    }
}

impl DerefMut for Tms {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self
    }
}
