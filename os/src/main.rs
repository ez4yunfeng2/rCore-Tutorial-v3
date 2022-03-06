#![no_std]
#![no_main]
#![feature(asm)]
#![feature(fn_traits)]
#![feature(global_asm)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

use k210_pac::Interrupt;
use k210_soc::{
    dmac::channel_interrupt_clear,
    plic::{plic_enable, set_priority, set_thershold},
    sysctl::dma_channel,
};

use crate::{
    sbi::sbi_rustsbi_k210_sext,
    sync::UPSafeCell,
};

extern crate alloc;

#[macro_use]
extern crate bitflags;

#[macro_use]
mod console;
mod config;
mod drivers;
mod fatfs;
mod fs;
mod lang_items;
mod mm;
mod sbi;
mod sync;
mod syscall;
mod task;
mod timer;
mod trap;
// mod irq;
global_asm!(include_str!("entry.asm"));

fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    unsafe {
        core::slice::from_raw_parts_mut(sbss as usize as *mut u8, ebss as usize - sbss as usize)
            .fill(0);
    }
}

#[no_mangle]
pub fn rust_main(hartid: usize) -> ! {
    if hartid == 0 {
        clear_bss();
        mm::init();
        mm::remap_test();
        trap::init();
        // trap::enable_timer_interrupt();
        // timer::set_next_trigger();
        // irq::irq_init();
        println!("[kernel] Lotus core {}", hartid);
        println!("{}", include_str!("banner"));
        {
            sbi_rustsbi_k210_sext();
            set_thershold(0);
            plic_enable(Interrupt::DMA0);
            set_priority(Interrupt::DMA0,1);
        }
        fatfs::fs_init();
        task::add_initproc();
        // send_ipi(1);
    } else {
        mm::activate();
        trap::init();
        trap::enable_timer_interrupt();
        timer::set_next_trigger();
        println!("Init hart 1 ");
        loop {}
    }

    unsafe { asm!("mv tp, {}",in(reg) hartid) }
    task::run_tasks(hartid);
    panic!("Unreachable in rust_main!");
}

lazy_static::lazy_static!(
    pub static ref TEMP:UPSafeCell<bool> = unsafe{ UPSafeCell::new(true) };
);

#[no_mangle]
pub fn wait_for_irq() {
    while *TEMP.exclusive_access() {}
    *TEMP.exclusive_access() = true;
}

pub unsafe fn handler_ext() {
    let ptr = k210_pac::PLIC::ptr();
    let irq = (*ptr).targets[0].claim.read().bits();
    match irq  {
        27 => {
            channel_interrupt_clear(dma_channel::CHANNEL0);
            *TEMP.exclusive_access() = false;
        }
        33 => {
            
        }
        _ => {
            panic!("unknow irq")
        }
    }
    (*ptr).targets[0].claim.write(|w|w.bits(irq));
}
