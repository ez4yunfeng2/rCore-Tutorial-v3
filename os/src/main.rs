#![no_std]
#![no_main]
#![feature(global_asm)]
#![feature(asm)]
#![feature(fn_traits)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

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
mod irq;
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
pub fn rust_main() -> ! {
    clear_bss();
    mm::init();
    mm::remap_test();
    println!("[kernel] Hello, world!");
    println!("{}",include_str!("banner"));
    irq::irq_init();
    trap::init();
    trap::enable_timer_interrupt();
    timer::set_next_trigger();
    fatfs::fs_init();
    task::add_initproc();
    task::run_tasks();
    panic!("Unreachable in rust_main!");
}
