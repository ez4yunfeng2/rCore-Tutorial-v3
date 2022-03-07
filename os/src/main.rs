#![no_std]
#![no_main]
#![feature(asm)]
#![feature(fn_traits)]
#![feature(global_asm)]
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
    if hartid == 0 {
        clear_bss();
        mm::init();
        mm::remap_test();
        trap::init();
        // trap::enable_timer_interrupt();
        // timer::set_next_trigger();

        println!("[kernel] Lotus core {}", hartid);
        println!("{}", include_str!("banner"));

        irq::irq_init();
        fatfs::fs_init();
        task::add_initproc();
    } else {
        mm::activate();
        trap::init();
        trap::enable_timer_interrupt();
        timer::set_next_trigger();
        println!("Init hart 1 ");
        loop {}
    }
    task::run_tasks(hartid);
    panic!("Unreachable in rust_main!");
}
