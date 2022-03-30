#![no_std]
#![no_main]
#![feature(asm)]
#![allow(unused)]
#![feature(fn_traits)]
#![feature(global_asm)]
#![feature(const_btree_new)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

use core::sync::atomic::{fence, Ordering};

use k210_pac::dmac::channel::status;
use riscv::register::{sie, sstatus};

use crate::{
    drivers::BLOCK_DEVICE,
    sbi::{sbi_smext_stimer, send_ipi},
    sync::{intr_off, intr_on},
};
extern crate alloc;
#[macro_use]
extern crate bitflags;
#[macro_use]
mod console;
#[macro_use]
mod config;
mod drivers;
mod fatfs;
mod fs;
mod irq;
mod lang_items;
mod mm;
mod sbi;
mod sync;
mod syscall;
mod task;
mod timer;
mod trap;
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
    unsafe { asm!("mv tp, {}", in(reg)hartid) }
    if hartid == 0 {
        clear_bss();
        mm::init();
        mm::remap_test();
        println!("[kernel] Lotus core {} boot", hartid);
        println!("{}", include_str!("banner"));
        console::logger_init();
        trap::init();
        irq::irq_init(hartid);
        trap::enable_timer_interrupt();
        timer::set_next_trigger();
        fatfs::fs_init();
        task::add_initproc();
        BLOCK_DEVICE.change_mode();
        send_ipi(1);
    } else {
        mm::activate();
        trap::init();
        irq::irq_init(hartid);
        trap::enable_timer_interrupt();
        timer::set_next_trigger();
        println!("[kernel] Lotus core {} boot", hartid);
    }
    task::run_tasks(hartid);
    panic!("Unreachable in rust_main!");
}
