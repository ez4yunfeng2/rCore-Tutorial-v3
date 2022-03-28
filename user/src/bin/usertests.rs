#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

static TESTS: &[&str] = &[
    "brk\0",
    "chdir\0",
    "clone\0",
    "close\0",
    "dup\0",
    "dup2\0",
    "execve\0",
    "exit\0",
    "fork\0",
    "fstat\0",
    "getcwd\0",
    "getdents\0",
    "getpid\0",
    "getppid\0",
    "gettimeofday\0",
    "mkdir\0",
    "mmap\0",
    "munmap\0",
    "mount\0",
    "open\0",
    "openat\0",
    "pipe\0",
    "read\0",
    "times\0",
    "umount\0",
    "uname\0",
    "wait\0",
    "yield\0",
    "write\0",
    "waitpid\0",
    "user_shell\0"
];

use user_lib::{exec, fork, waitpid};

#[no_mangle]
pub fn main() -> i32 {
    for test in TESTS {
        println!("Usertests: Running {}", test);
        let pid = fork();
        if pid == 0 {
            exec(*test, &[0 as *const u8]);
            panic!("unreachable!");
        } else {
            let mut exit_code: i32 = Default::default();
            let wait_pid = waitpid(pid as usize, &mut exit_code);
            assert_eq!(pid, wait_pid);
            println!("\x1b[32mUsertests: Test {} in Process {} exited with code {}\x1b[0m", test, pid, exit_code);
        }
    }
    println!("Usertests passed!");
    0
}