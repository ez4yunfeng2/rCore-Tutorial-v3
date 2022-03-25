mod context;
use crate::config::TRAMPOLINE;
use crate::irq::handler_ext;
use crate::sbi::sbi_smext_stimer;
use crate::sync::{intr_off, intr_on};
use crate::syscall::syscall;
use crate::task::{
    current_hartid, current_trap_cx, current_trap_cx_user_va, current_user_token, enter_kernel,
    exit_current_and_run_next, exit_kernel, suspend_current_and_run_next,
};
use crate::timer::{check_timer, get_time, get_time_usec, set_next_trigger};
pub use context::TrapContext;
use log::{info, trace};
use riscv::register::sepc;
use riscv::register::sstatus::{self, SPP};
use riscv::register::{
    mtvec::TrapMode,
    scause::{self, Exception, Interrupt, Trap},
    sie, stval, stvec,
};

global_asm!(include_str!("trap.S"));

pub fn init() {
    set_kernel_trap_entry();
}

fn set_kernel_trap_entry() {
    unsafe {
        extern "C" {
            fn __from_kernel_save();
        }
        stvec::write(__from_kernel_save as usize, TrapMode::Direct);
    }
}

fn set_user_trap_entry() {
    unsafe {
        stvec::write(TRAMPOLINE as usize, TrapMode::Direct);
    }
}

pub fn enable_timer_interrupt() {
    unsafe {
        sie::set_stimer();
    }
}

#[no_mangle]
pub fn trap_handler() -> ! {
    debug_assert_eq!(sstatus::read().sie(), false, "Trap sie enabled");
    debug_assert_eq!(sstatus::read().spp(), SPP::User, "Trop from kernel");
    set_kernel_trap_entry();
    let scause = scause::read();
    let stval = stval::read();
    match scause.cause() {
        Trap::Exception(Exception::UserEnvCall) => {
            enter_kernel();
            // jump to next instruction anyway
            let mut cx = current_trap_cx();
            cx.sepc += 4;
            // intr_on();
            // get system call return value
            let result = syscall(
                cx.x[17],
                [cx.x[10], cx.x[11], cx.x[12], cx.x[13], cx.x[14], cx.x[15]],
            );
            // cx is changed during sys_exec, so we have to call it again
            cx = current_trap_cx();
            cx.x[10] = result as usize;
        }
        Trap::Exception(Exception::StoreFault)
        | Trap::Exception(Exception::StorePageFault)
        | Trap::Exception(Exception::InstructionFault)
        | Trap::Exception(Exception::InstructionPageFault)
        | Trap::Exception(Exception::LoadFault)
        | Trap::Exception(Exception::LoadPageFault) => {
            println!(
                "[kernel] {:?} in application, bad addr = {:#x}, bad instruction = {:#x}, core dumped.",
                scause.cause(),
                stval,
                current_trap_cx().sepc,
            );
            // page fault exit code
            exit_current_and_run_next(-2);
        }
        Trap::Exception(Exception::IllegalInstruction) => {
            println!("[kernel] IllegalInstruction in application, core dumped.");
            // illegal instruction exit code
            exit_current_and_run_next(-3);
        }
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            set_next_trigger();
            if current_hartid() == 0 {
                check_timer();
            }
            suspend_current_and_run_next();
        }
        Trap::Interrupt(Interrupt::SupervisorSoft) => {
            // info!("U SupervisorSoft");
            handler_ext();
            sbi_smext_stimer();
        }
        _ => {
            panic!(
                "Unsupported U trap {:?}, stval = {:#x}!",
                scause.cause(),
                stval
            );
        }
    }
    trap_return();
}

#[no_mangle]
pub fn trap_return() -> ! {
    intr_off();
    set_user_trap_entry();
    exit_kernel();
    let trap_cx_user_va = current_trap_cx_user_va();
    let user_satp = current_user_token();
    extern "C" {
        fn __alltraps();
        fn __restore();
    }
    let restore_va = __restore as usize - __alltraps as usize + TRAMPOLINE;
    
    unsafe {
        asm!(
            "fence.i",
            "jr {restore_va}",
            restore_va = in(reg) restore_va,
            in("a0") trap_cx_user_va,
            in("a1") user_satp,
            options(noreturn)
        );
    }
}

#[no_mangle]
pub fn trap_from_kernel() {
    assert_eq!(sstatus::read().spp(), SPP::Supervisor);
    debug_assert_eq!(sstatus::read().sie(), false, "Trap_from_kernel sie enabled");
    let scause = scause::read();
    let stval = stval::read();
    let sepc = sepc::read();
    match scause.cause() {
        Trap::Interrupt(Interrupt::SupervisorSoft) => {
            handler_ext();
            sbi_smext_stimer();
        }
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            set_next_trigger();
            if current_hartid() == 0 {
                check_timer();
            }
            // suspend_current_and_run_next();
        }
        Trap::Interrupt(_) => todo!(),
        Trap::Exception(_) => {
            panic!(
                "Unsupported S trap {:?}, stval = {:#x}, sepc = {:#x}, {:#x}",
                scause.cause(),
                stval,
                sepc,
                unsafe { *(stval as *const u64) }
            );
        }
    }
}
