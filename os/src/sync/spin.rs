use core::sync::atomic::{compiler_fence, fence, AtomicI8, AtomicIsize, AtomicUsize};
#[allow(unused)]
use core::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicBool, Ordering},
};

use riscv::register::sstatus;

use crate::task::{current_hartid, current_processor};

pub struct SpinMutex<T: ?Sized> {
    pub(crate) lock: AtomicBool,
    pub(crate) cpu: AtomicIsize,
    data: UnsafeCell<T>,
}

unsafe impl<T> Sync for SpinMutex<T> {}
unsafe impl<T> Send for SpinMutex<T> {}
pub struct SpinMutexGuard<'a, T: ?Sized + 'a> {
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

impl<T: ?Sized> SpinMutex<T> {
    #[inline(always)]
    pub fn is_locked(&self) -> bool {
        self.lock.load(Ordering::Relaxed)
    }

    pub fn holding(&self) -> bool {
        self.lock.load(Ordering::Relaxed)
            && self.cpu.load(Ordering::Relaxed) == current_hartid() as isize
    }

    #[inline(always)]
    pub fn lock(&self) -> SpinMutexGuard<T> {
        push_off();
        if self.holding() {
            panic!("acquire")
        }
        let mut timeout = 0x0usize;
        while self
            .lock
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            while self.is_locked() {
                timeout+=1;
                if timeout >= 0xFFFFFF{
                    panic!("lock timeout")
                }
            }
        }
        self.cpu.swap(current_hartid() as isize, Ordering::Relaxed);
        SpinMutexGuard {
            lock: &self.lock,
            cpu: &self.cpu,
            data: unsafe { &mut *self.data.get() },
        }
    }

    #[inline(always)]
    pub fn try_lock(&self) -> Option<SpinMutexGuard<T>> {
        push_off();
        if self.holding() {
           panic!("acquire")
        }
        if self.lock.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed).is_ok() {
            self.cpu.swap(current_hartid() as isize, Ordering::Relaxed);
            Some(SpinMutexGuard {
                lock: &self.lock,
                cpu: &self.cpu,
                data: unsafe { &mut *self.data.get() },
            })
        } else {
            None
        }
    }
}

impl<T> From<T> for SpinMutex<T> {
    fn from(data: T) -> Self {
        Self::new(data)
    }
}

impl<'a, T: ?Sized> Deref for SpinMutexGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.data
    }
}

impl<'a, T: ?Sized> DerefMut for SpinMutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.data
    }
}

impl<'a, T: ?Sized> Drop for SpinMutexGuard<'a, T> {
    fn drop(&mut self) {
        if !self.holding() {
            panic!("release")
        }
        self.cpu.store(-1, Ordering::Release);
        self.lock.store(false, Ordering::Release);
        pop_off();
    }
}

impl<'a, T: ?Sized> SpinMutexGuard<'a, T> {
    pub fn holding(&self) -> bool {
        self.lock.load(Ordering::Relaxed)
            && self.cpu.load(Ordering::Relaxed) == current_hartid() as isize
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

pub fn push_off() {
    intr_off();
    let processor = current_processor().unwrap();
    let old = sstatus::read().sie();
    if processor.noff == 0 {
        processor.intena = old
    }
    processor.noff += 1;
}

pub fn pop_off() {
    let processor = current_processor().unwrap();
    if sstatus::read().sie() {
        panic!("pop_off - interruptible")
    }
    if processor.noff < 1 {
        println!("pop_off {} {}", processor.noff < 1, processor.noff);
        panic!("pop_off")
    }
    processor.noff -= 1;
    if processor.noff == 0 && processor.intena {
        intr_on()
    }
}
