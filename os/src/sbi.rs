#![allow(unused)]
const SBI_SET_TIMER: usize = 0;
const SBI_CONSOLE_PUTCHAR: usize = 1;
const SBI_CONSOLE_GETCHAR: usize = 2;
const SBI_CLEAR_IPI: usize = 3;
const SBI_SEND_IPI: usize = 4;
const SBI_REMOTE_FENCE_I: usize = 5;
const SBI_REMOTE_SFENCE_VMA: usize = 6;
const SBI_REMOTE_SFENCE_VMA_ASID: usize = 7;
const SBI_SHUTDOWN: usize = 8;
const LEGACY_SEND_IPI: usize = 4;
#[inline(always)]
fn sbi_call(which: usize, arg0: usize, arg1: usize, arg2: usize) -> usize {
    let mut ret;
    unsafe {
        asm!(
            "ecall",
            inlateout("x10") arg0 => ret,
            in("x11") arg1,
            in("x12") arg2,
            in("x17") which,
        );
    }
    ret
}

pub fn set_timer(timer: usize) {
    sbi_call(SBI_SET_TIMER, timer, 0, 0);
}

pub fn console_putchar(c: usize) {
    sbi_call(SBI_CONSOLE_PUTCHAR, c, 0, 0);
}

pub fn console_getchar() -> usize {
    loop {
        let ch = sbi_call(SBI_CONSOLE_GETCHAR, 0, 0, 0);
        if (ch as u8 as char).is_ascii() {
            return ch;
        }
    }
}

pub fn send_ipi(hartid: usize) {
    let hartid_mask = 1usize << hartid;
    sbi_call(LEGACY_SEND_IPI, &hartid_mask as *const usize as usize, 0, 0);
}

// #[inline]
// pub fn sbi_rustsbi_k210_sext() {
//     sbi_call(0x0A000004, handler_ext as usize, 0, 0);
// }

pub fn sbi_smext_stimer() {
    sbi_call(1225, 0, 0, 0);
}

pub fn shutdown() -> ! {
    loop {}
    // sbi_call(SBI_SHUTDOWN, 0, 0, 0);
}
