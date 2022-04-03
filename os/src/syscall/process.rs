use crate::fs::{open_file, OpenFlags};
use crate::mm::{translated_ref, translated_refmut, translated_str};
use crate::task::{
    current_hartid, current_process, current_task, current_user_token, exit_current_and_run_next,
    suspend_current_and_run_next, Tms,
};
use crate::timer::get_time_ms;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use riscv::register::sstatus;

pub fn sys_exit(exit_code: i32) -> ! {
    exit_current_and_run_next(exit_code);
    panic!("Unreachable in sys_exit!");
}

pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}

pub fn sys_getpid() -> isize {
    current_task().unwrap().process.upgrade().unwrap().getpid() as isize
}

pub fn sys_getppid() -> isize {
    current_task().unwrap().process.upgrade().unwrap().getppid() as isize
}

pub fn sys_fork() -> isize {
    // println!("sys_fork1");
    println!("[info] fork");
    let current_process = current_process();
    let new_process = current_process.fork();
    println!("[info] fork2");
    let new_pid = new_process.getpid();
    
    // modify trap context of new_task, because it returns immediately after switching
    let new_process_inner = new_process.inner_lock_access().unwrap();
    let task = new_process_inner.tasks[0].as_ref().unwrap();
    let trap_cx = task.inner_lock_access().unwrap().get_trap_cx();
    // we do not have to move to next instruction since we have done it before
    // for child process, fork returns 0
    trap_cx.x[10] = 0;
    new_pid as isize
}

pub fn sys_exec(path: *const u8, mut args: *const usize) -> isize {
    let token = current_user_token();
    let path = translated_str(token, path);
    let mut args_vec: Vec<String> = Vec::new();
    loop {
        let arg_str_ptr = *translated_ref(token, args);
        if arg_str_ptr == 0 {
            break;
        }
        args_vec.push(translated_str(token, arg_str_ptr as *const u8));
        unsafe {
            args = args.add(1);
        }
    }
    if let Some(app_inode) = open_file(path.as_str(), OpenFlags::RDONLY) {
        let all_data = app_inode.read_all();
        let process = current_process();
        let argc = args_vec.len();
        process.exec(all_data.as_slice(), args_vec);
        // return argc because cx.x[10] will be covered with it later
        argc as isize
    } else {
        -1
    }
}

/// If there is not a child process whose pid is same as given, return -1.
/// Else if there is a child process but it is still running, return -2.
pub fn sys_waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
    println!("waitpid");
    let process = current_process();
    let inner = process.inner_lock_access().unwrap();
    if inner
        .children
        .iter()
        .find(|p| pid == -1 || pid as usize == p.getpid())
        .is_none()
    {
        return -1;
    }
    drop(inner);
    drop(process);
    loop {
        intr_check!();
        let process = current_process();
        let mut inner = process.inner_lock_access().unwrap();
        if let Some((idx, _)) = inner.children.iter().enumerate().find(|(_, p)| {
            let tmp = p.getpid();
            match p.try_inner_exclusive_access() {
                Ok(p) => p.is_zombie && (pid == -1 || pid as usize == tmp),
                Err(_) => false,
            }
        }) {
            let child = inner.children.remove(idx);
            assert_eq!(Arc::strong_count(&child), 1);
            let found_pid = child.getpid();
            let exit_code = child.inner_lock_access().unwrap().exit_code;
            if exit_code_ptr as usize != 0 {
                *translated_refmut(inner.memory_set.token(), exit_code_ptr) = 3 << 8 | exit_code;
            }
            return found_pid as isize;
        }
        drop(inner);
        drop(process);
        suspend_current_and_run_next()
    }
}

pub fn sys_brk(addr: usize) -> isize {
    let task = current_task().unwrap();
    let mut inner = task.inner_lock_access().unwrap();
    if let Some(res) = inner.res.as_mut() {
        res.brk(addr) as isize
    } else {
        -1
    }
}

pub fn sys_times(tms: *mut Tms) -> isize {
    let token = current_user_token();
    let tms = translated_refmut(token, tms);
    let task = current_task().unwrap();
    let mut inner = task.inner_lock_access().unwrap();
    *tms = *inner.times;
    1
}