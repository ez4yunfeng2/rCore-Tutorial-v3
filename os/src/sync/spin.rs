#[allow(unused)]
use core::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicBool, Ordering},
};
use core::{
    fmt::Debug,
    sync::atomic::{compiler_fence, fence, AtomicI8, AtomicIsize, AtomicUsize},
};
use riscv::register::sstatus;

use crate::task::{current_hartid, current_processor};
#[derive(Debug)]
pub enum LockError {
    Release,
    Hold,
    UnknowProc,
    Interruptible,
    LockFail,
    Unknow
}

pub struct SpinMutex<T: ?Sized> {
    pub(crate) lock: AtomicBool,
    pub(crate) cpu: AtomicIsize,
    data: UnsafeCell<T>,
}

unsafe impl<T> Sync for SpinMutex<T> {}
unsafe impl<T> Send for SpinMutex<T> {}
#[derive(Debug)]
pub struct SpinMutexGuard<'a, T: Debug + ?Sized + 'a> {
    lock: &'a AtomicBool,
    cpu: &'a AtomicIsize,
    data: &'a mut T,
}

impl<T> SpinMutex<T> {
    #[inline(always)]
    pub const fn new(user_data: T) -> SpinMutex<T> {
        SpinMutex {
            lock: AtomicBool::new(false),
            cpu: AtomicIsize::new(-1),
            data: UnsafeCell::new(user_data),
        }
    }
}

impl<T: ?Sized + Debug> SpinMutex<T> {

    #[inline(always)]
    pub fn is_locked(&self) -> bool {
        self.lock.load(Ordering::Relaxed)
    }

    pub fn holding(&self) -> bool {
        self.lock.load(Ordering::Relaxed)
            && self.cpu.load(Ordering::Relaxed) == current_hartid() as isize
    }

    pub fn check_cpu(&self) -> bool {
        self.cpu.load(Ordering::Relaxed) == current_hartid() as isize
    }

    pub fn check_hold(&self) -> bool {
        self.lock.load(Ordering::Relaxed)
    }

    #[inline(always)]
    pub fn lock(&self) -> Result<SpinMutexGuard<T>, LockError> {
        intr_off();
        if self.check_cpu() {
            return Err(LockError::UnknowProc);
        }
        if self.check_hold() {
            return Err(LockError::Hold);
        }
        while self
            .lock
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            while self.is_locked() {}
        }
        self.cpu.swap(current_hartid() as isize, Ordering::Relaxed);
        Ok(SpinMutexGuard {
            lock: &self.lock,
            cpu: &self.cpu,
            data: unsafe { &mut *self.data.get() },
        })
    }

    #[inline(always)]
    pub fn try_lock(&self) -> Result<SpinMutexGuard<T>, LockError> {
        intr_off();
        if self
            .lock
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
        {
            self.cpu.swap(current_hartid() as isize, Ordering::Relaxed);
            Ok(SpinMutexGuard {
                lock: &self.lock,
                cpu: &self.cpu,
                data: unsafe { &mut *self.data.get() },
            })
        } else {
            Err(LockError::LockFail)
        }
    }
}

impl<T> From<T> for SpinMutex<T> {
    fn from(data: T) -> Self {
        Self::new(data)
    }
}

impl<'a, T: ?Sized + Debug> Deref for SpinMutexGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.data
    }
}

impl<'a, T: ?Sized + Debug> DerefMut for SpinMutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.data
    }
}

impl<'a, T: ?Sized + Debug> Drop for SpinMutexGuard<'a, T> {
    fn drop(&mut self) {
        if !self.check_cpu() {
            panic!("release wrong cpu {}",self.cpu.load(Ordering::Relaxed))
        }
        if !self.check_hold() {
            panic!("release no hold")
        }
        self.cpu.store(-1, Ordering::Release);
        self.lock.store(false, Ordering::Release);
        // pop_off().unwrap();
    }
}

impl<'a, T: ?Sized + Debug> SpinMutexGuard<'a, T> {
    pub fn holding(&self) -> bool {
        self.lock.load(Ordering::Relaxed)
            && self.cpu.load(Ordering::Relaxed) == current_hartid() as isize
    }
    pub fn check_cpu(&self) -> bool {
        self.cpu.load(Ordering::Relaxed) == current_hartid() as isize
    }

    pub fn check_hold(&self) -> bool {
        self.lock.load(Ordering::Relaxed)
    }
}

#[inline]
pub fn intr_on() {
    unsafe {
        sstatus::set_sie();
    }
}

#[inline]
pub fn intr_off() {
    unsafe {
        sstatus::clear_sie();
    }
}