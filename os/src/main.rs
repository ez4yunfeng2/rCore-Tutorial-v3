#![no_std]
#![no_main]
#![feature(global_asm)]
#![feature(asm)]
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
    println!("[kernel] Hello, world!");
    println!("{}",include_str!("banner"));
    // {
    //     // use k210_soc::plic;
    //     use k210_soc::gpiohs;
    //     use k210_soc::gpio::{GpioPinEdge,direction};
    //     use k210_soc::fpioa;
    //     // plic::enable(plic::interrupt::GPIOHS0);
    //     // plic::set_priority(plic::interrupt::GPIOHS0, 1);
    //     // plic::set_thershold(0);
    //     gpiohs::set_pin_edge(0, GpioPinEdge::GPIO_PE_LOW);
    //     fpioa::set_function(fpioa::io::LED_B , fpioa::function::GPIOHS0);
    //     gpiohs::set_direction(0, direction::OUTPUT);
    //     gpiohs::set_pin(0, false);

    // }
    mm::init();
    mm::remap_test();
    trap::init();
    trap::enable_timer_interrupt();
    timer::set_next_trigger();
    fatfs::fs_init();
    task::add_initproc();
    task::run_tasks();
    panic!("Unreachable in rust_main!");
}
