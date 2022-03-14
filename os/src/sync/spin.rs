#[allow(unused)]
use core::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicBool, Ordering},
};

use riscv::register::sstatus;

use crate::task::current_processor;

pub struct SpinMutex<T: ?Sized> {
    pub(crate) lock: AtomicBool,
    data: UnsafeCell<T>,
}

unsafe impl<T> Sync for SpinMutex<T> {}
unsafe impl<T> Send for SpinMutex<T> {}
pub struct SpinMutexGuard<'a, T: ?Sized + 'a> {
    lock: &'a AtomicBool,
    data: &'a mut T,
}

impl<T> SpinMutex<T> {
    #[inline(always)]
    pub const fn new(user_data: T) -> SpinMutex<T> {
        SpinMutex {
            lock: AtomicBool::new(false),
            data: UnsafeCell::new(user_data),
        }
    }
}

impl<T: ?Sized> SpinMutex<T> {
    #[inline(always)]
    pub fn is_locked(&self) -> bool {
        self.lock.load(Ordering::Relaxed)
    }

    #[inline(always)]
    pub fn lock(&self) -> SpinMutexGuard<T> {
        push_off();
        while self
            .lock
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            while self.is_locked() {}
        }

        SpinMutexGuard {
            lock: &self.lock,
            data: unsafe { &mut *self.data.get() },
        }
    }

    #[inline(always)]
    pub unsafe fn force_unlock(&self) {
        self.lock.store(false, Ordering::Release);
    }
    #[inline(always)]
    pub fn try_lock(&self) -> Option<SpinMutexGuard<T>> {
        // The reason for using a strong compare_exchange is explained here:
        // https://github.com/Amanieu/parking_lot/pull/207#issuecomment-575869107
        if self
            .lock
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
        {
            Some(SpinMutexGuard {
                lock: &self.lock,
                data: unsafe { &mut *self.data.get() },
            })
        } else {
            None
        }
    }
    #[inline(always)]
    pub fn get_mut(&mut self) -> &mut T {
        // We know statically that there are no other references to `self`, so
        // there's no need to lock the inner mutex.
        unsafe { &mut *self.data.get() }
    }
}

impl<T> From<T> for SpinMutex<T> {
    fn from(data: T) -> Self {
        Self::new(data)
    }
}

impl<'a, T: ?Sized> SpinMutexGuard<'a, T> {
    #[inline(always)]
    pub fn leak(this: Self) -> &'a mut T {
        let data = this.data as *mut _; // Keep it in pointer form temporarily to avoid double-aliasing
        core::mem::forget(this);
        unsafe { &mut *data }
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
        self.lock.store(false, Ordering::Release);
        pop_off();
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
    let processor = current_processor().unwrap();
    let old = sstatus::read().sie();
    intr_off();
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
        panic!("pop_off {}", processor.noff)
    }
    processor.noff -= 1;
    if processor.noff == 0 && processor.intena {
        intr_on()
    }
}
