use crate::{
    mm::translated_refmut,
    task::current_user_token,
    timer::{get_time_sec, get_time_usec},
};
const SYSNAME: &[u8; 7] = b"LotusOs";
const NODENAME: &[u8; 7] = b"lotusOS";
const RELEASE: &[u8; 18] = b"2020/2/14 23:13:35";
const VERSION: &[u8; 6] = b"V0.0.1";
const MACHINE: &[u8; 9] = b"riscv64gc";
const DOMAINNAME: &[u8; 9] = b"localhost";
#[repr(C)]
pub struct Utsname {
    sysname: [u8; 65],
    nodename: [u8; 65],
    release: [u8; 65],
    version: [u8; 65],
    machine: [u8; 65],
    domainname: [u8; 65],
}

pub fn sys_uname(ptr: *mut Utsname) -> isize {
    let token = current_user_token();
    let uname = translated_refmut(token, ptr);
    uname.sysname[0..SYSNAME.len()].copy_from_slice(SYSNAME);
    uname.nodename[0..NODENAME.len()].copy_from_slice(NODENAME);
    uname.release[0..RELEASE.len()].copy_from_slice(RELEASE);
    uname.machine[0..MACHINE.len()].copy_from_slice(MACHINE);
    uname.version[0..VERSION.len()].copy_from_slice(VERSION);
    uname.domainname[0..DOMAINNAME.len()].copy_from_slice(DOMAINNAME);
    0
}

#[repr(C)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

pub fn sys_get_time(ptr: *mut TimeVal) -> isize {
    let token = current_user_token();
    let mut time = translated_refmut(token, ptr);
    time.sec = get_time_sec();
    time.usec = get_time_usec();
    0
}
