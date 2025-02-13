const SYSCALL_GET_CWD: usize = 17;
const SYSCALL_DUP: usize = 23;
const SYSCALL_DUP3: usize = 24;
const SYSCALL_MKDIR: usize = 34;
const SYSCALL_UNLINK: usize = 35;
const SYSCALL_UMOUNT: usize = 39;
const SYSCALL_MOUNT: usize = 40;
const SYSCALL_CHDIR: usize = 49;
const SYSCALL_OPEN: usize = 56;
const SYSCALL_CLOSE: usize = 57;
const SYSCALL_PIPE: usize = 59;
const SYSCALL_GETDENTS: usize = 61;
const SYSCALL_READ: usize = 63;
const SYSCALL_WRITE: usize = 64;
const SYSCALL_FSTAT: usize = 80;
const SYSCALL_EXIT: usize = 93;
const SYSCALL_SLEEP: usize = 101;
const SYSCALL_YIELD: usize = 124;
const SYSCALL_UNAME: usize = 160;
const SYSCALL_GET_TIME: usize = 169;
const SYSCALL_GETPID: usize = 172;
const SYSCALL_GETPPID: usize = 173;
const SYSCALL_BRK: usize = 214;
const SYSCALL_MUNMAP: usize = 215;
const SYSCALL_FORK: usize = 220;
const SYSCALL_EXEC: usize = 221;
const SYSCALL_MMAP: usize = 222;
const SYSCALL_WAITPID: usize = 260;
const SYSCALL_THREAD_CREATE: usize = 1000;
const SYSCALL_GETTID: usize = 1001;
const SYSCALL_WAITTID: usize = 1002;
const SYSCALL_MUTEX_CREATE: usize = 1010;
const SYSCALL_MUTEX_LOCK: usize = 1011;
const SYSCALL_MUTEX_UNLOCK: usize = 1012;
const SYSCALL_SEMAPHORE_CREATE: usize = 1020;
const SYSCALL_SEMAPHORE_UP: usize = 1021;
const SYSCALL_SEMAPHORE_DOWN: usize = 1022;

mod fs;
mod osinfo;
mod process;
mod sync;
mod thread;

use fs::*;
use osinfo::*;
use process::*;
use sync::*;
use thread::*;

use crate::fs::{Dirent, Kstat};

use self::osinfo::TimeVal;

pub fn syscall(syscall_id: usize, args: [usize; 6]) -> isize {
    match syscall_id {
        SYSCALL_GET_CWD => sys_getcwd(args[0] as *const u8, args[1]),
        SYSCALL_DUP => sys_dup(args[0]),
        SYSCALL_DUP3 => sys_dup3(args[0], args[1]),
        SYSCALL_MKDIR => sys_mkdir(args[0] as isize, args[1] as *const u8, args[2]),
        SYSCALL_UMOUNT => sys_umount(),
        SYSCALL_MOUNT => sys_mount(),
        SYSCALL_UNLINK => sys_unlink(args[0] as isize, args[1] as *const u8, args[2]),
        SYSCALL_CHDIR => sys_chdir(args[0] as *const u8),
        SYSCALL_OPEN => sys_open(args[0] as isize, args[1] as *const u8, args[2] as u32),
        SYSCALL_CLOSE => sys_close(args[0]),
        SYSCALL_PIPE => sys_pipe(args[0] as *mut i32),
        SYSCALL_GETDENTS => sys_getdents(args[0] as isize, args[1] as *mut Dirent, args[2]),
        SYSCALL_READ => sys_read(args[0], args[1] as *const u8, args[2]),
        SYSCALL_WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        SYSCALL_FSTAT => sys_fstat(args[0] as isize, args[1] as *mut Kstat),
        SYSCALL_EXIT => sys_exit(args[0] as i32),
        SYSCALL_SLEEP => sys_sleep(args[0]),
        SYSCALL_YIELD => sys_yield(),
        SYSCALL_UNAME => sys_uname(args[0] as *mut Utsname),
        SYSCALL_GET_TIME => sys_get_time(args[0] as *mut TimeVal),
        SYSCALL_GETPID => sys_getpid(),
        SYSCALL_GETPPID => sys_getppid(),
        SYSCALL_BRK => sys_brk(args[0]),
        SYSCALL_MUNMAP => sys_munmap(args[0], args[1]),
        SYSCALL_FORK => sys_fork(),
        SYSCALL_EXEC => sys_exec(args[0] as *const u8, args[1] as *const usize),
        SYSCALL_MMAP => sys_mmap(args[0], args[1], args[2], args[3], args[4], args[5]),
        SYSCALL_WAITPID => sys_waitpid(args[0] as isize, args[1] as *mut i32),
        SYSCALL_THREAD_CREATE => sys_thread_create(args[0], args[1]),
        SYSCALL_GETTID => sys_gettid(),
        SYSCALL_WAITTID => sys_waittid(args[0]) as isize,
        SYSCALL_MUTEX_CREATE => sys_mutex_create(args[0] == 1),
        SYSCALL_MUTEX_LOCK => sys_mutex_lock(args[0]),
        SYSCALL_MUTEX_UNLOCK => sys_mutex_unlock(args[0]),
        SYSCALL_SEMAPHORE_CREATE => sys_semaphore_creare(args[0]),
        SYSCALL_SEMAPHORE_UP => sys_semaphore_up(args[0]),
        SYSCALL_SEMAPHORE_DOWN => sys_semaphore_down(args[0]),
        _ => panic!("Unsupported syscall_id: {}", syscall_id),
    }
}
