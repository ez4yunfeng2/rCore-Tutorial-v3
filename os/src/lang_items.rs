use crate::sbi::shutdown;
use crate::sync::intr_off;
use crate::task::{current_hartid, current_kstack_top, current_process};
use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    intr_off();
    match info.location() {
        Some(location) => {
            println!(
                "[LotusOS] hartid {} panicked at '{}', {}:{}:{}, {}",
                current_hartid(),
                info.message().unwrap(),
                location.file(),
                location.line(),
                location.column(),
                current_process().getpid()
            );
        }
        None => println!(
            "[kernel] hartid {} panicked at '{}'",
            current_hartid(),
            info.message().unwrap()
        ),
    };
    unsafe {
        backtrace();
    }
    shutdown()
}

#[allow(unused)]
unsafe fn backtrace() {
    let mut fp: usize;
    let stop = current_kstack_top();
    asm!("mv {}, s0", out(reg) fp);
    println!("---START BACKTRACE---");
    for i in 0..10 {
        if fp == stop {
            break;
        }
        println!("#{}:ra={:#x}", i, *((fp - 8) as *const usize));
        fp = *((fp - 16) as *const usize);
    }
    println!("---END   BACKTRACE---");
}
