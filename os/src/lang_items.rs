use crate::sbi::shutdown;
use crate::task::current_hartid;
use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    match info.location() {
        Some(location) => {
            println!(
                "[kernel] hartid {} panicked at '{}', {}:{}:{}",
                current_hartid(),
                info.message().unwrap(),
                location.file(),
                location.line(),
                location.column()
            );
        }
        None => println!("[kernel] hartid {} panicked at '{}'", current_hartid(), info.message().unwrap()),
    }
    // unsafe {
    //     backtrace();
    // }
    shutdown()
}

// unsafe fn backtrace() {
//     let mut fp: usize;
//     let stop = current_kstack_top();
//     asm!("mv {}, s0", out(reg) fp);
//     println!("---START BACKTRACE---");
//     for i in 0..10 {
//         if fp == stop {
//             break;
//         }
//         println!("#{}:ra={:#x}", i, *((fp - 8) as *const usize));
//         fp = *((fp - 16) as *const usize);
//     }
//     println!("---END   BACKTRACE---");
// }
