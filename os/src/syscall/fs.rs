use super::process::sys_brk;
use crate::fatfs::io::SeekFrom;
use crate::fs::make_pipe;
use crate::fs::Dirent;
use crate::fs::Kstat;
use crate::fs::OpenFlags;
use crate::mm::{translated_byte_buffer, translated_refmut, translated_str, UserBuffer};
use crate::task::{current_process, current_user_token};
use alloc::string::ToString;
use alloc::sync::Arc;

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    let token = current_user_token();
    let process = current_process();
    let inner = process.inner_lock_access();
    if !inner.fd_table.contains_key(&fd) {
        return -1;
    }
    if let Some(file) = &inner.fd_table.get(&fd).unwrap() {
        if !file.writable() {
            return -1;
        }
        let file = file.clone();
        drop(inner);
        file.write(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
    } else {
        -1
    }
}

pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    let token = current_user_token();
    let process = current_process();
    let inner = process.inner_lock_access();
    if !inner.fd_table.contains_key(&fd) {
        return -1;
    }

    if let Some(file) = &inner.fd_table.get(&fd).unwrap() {
        let file = file.clone();
        if !file.readable() {
            return -1;
        }
        // release current task TCB manually to avoid multi-borrow
        drop(inner);
        file.read(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
    } else {
        -1
    }
}

pub fn sys_open(fd: isize, path: *const u8, flags: u32) -> isize {
    let token = current_user_token();
    let process = current_process();
    let mut inner = process.inner_lock_access();
    let path = translated_str(token, path).replace("./", "");
    println!("[sys_open]: {}", path);
    let dir = if fd >= 0 {
        let fd = fd as usize;
        if let Some(dir) = inner.fd_table.get(&fd).unwrap() {
            dir
        } else {
            return -1;
        }
    } else {
        &inner.dir_entry.as_ref().unwrap()
    };

    if path == ".".to_string() {
        let tmp = dir.clone();
        drop(dir);
        let fd = inner.alloc_fd();
        inner.fd_table.insert(fd, Some(tmp));
        return fd as isize;
    }
    let flag = OpenFlags::from_bits(flags).unwrap();
    let (readable, writable) = flag.read_write();
    let file = if flag.contains(OpenFlags::CREATE) {
        dir.create(&path, readable, writable, false)
    } else {
        let directory = flag.contains(OpenFlags::DIRECTORY);
        dir.open(&path, readable, writable, directory)
    };
    if let Some(file) = file {
        let fd = inner.alloc_fd();
        inner.fd_table.insert(fd, Some(file));
        fd as isize
    } else {
        -1
    }
}

pub fn sys_unlink(dirfd: isize, path: *const u8, _flags: usize) -> isize {
    let token = current_user_token();
    let process = current_process();
    let inner = process.inner_lock_access();
    if dirfd < 0 {
        let path = translated_str(token, path).replace("./", "");
        if inner.dir_entry.as_ref().unwrap().remove(&path) {
            return 0;
        }
    }
    1
}

pub fn sys_mkdir(dirfd: isize, path: *const u8, _mode: usize) -> isize {
    let token = current_user_token();
    let process = current_process();
    let mut inner = process.try_inner_exclusive_access().unwrap();
    let path = translated_str(token, path).replace("./", "");
    let dir = if dirfd >= 0 {
        if let Some(dir) = inner.fd_table.get(&(dirfd as usize)).unwrap() {
            dir
        } else {
            return -1;
        }
    } else {
        &inner.dir_entry.as_ref().unwrap()
    };

    if let Some(file) = dir.create(&path, false, false, true) {
        let fd = inner.alloc_fd();
        inner.fd_table.insert(fd, Some(file));
        fd as isize
    } else {
        -1
    }
}

pub fn sys_chdir(path: *const u8) -> isize {
    let token = current_user_token();
    let process = current_process();
    let mut inner = process.try_inner_exclusive_access().unwrap();
    let path = translated_str(token, path).replace("./", "");
    let inode = inner
        .dir_entry
        .as_ref()
        .unwrap()
        .open(&path, true, true, true);
    match inode {
        Some(file) => {
            inner.dir_entry = Some(file);
            0
        }
        None => 1,
    }
}

pub fn sys_close(fd: usize) -> isize {
    let process = current_process();
    let mut inner = process.try_inner_exclusive_access().unwrap();
    if !inner.fd_table.contains_key(&fd) {
        return -1;
    }
    inner.fd_table.remove(&fd);
    0
}

pub fn sys_pipe(pipe: *mut i32) -> isize {
    let process = current_process();
    let token = current_user_token();
    let mut inner = process.try_inner_exclusive_access().unwrap();
    let (pipe_read, pipe_write) = make_pipe();
    let read_fd = inner.alloc_fd();
    inner.fd_table.get_mut(&read_fd).unwrap().replace(pipe_read);
    let write_fd = inner.alloc_fd();
    inner
        .fd_table
        .get_mut(&write_fd)
        .unwrap()
        .replace(pipe_write);
    *translated_refmut(token, pipe) = read_fd as i32;
    *translated_refmut(token, unsafe { pipe.add(1) }) = write_fd as i32;
    0
}

pub fn sys_dup(fd: usize) -> isize {
    let process = current_process();
    let mut inner = process.try_inner_exclusive_access().unwrap();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if inner.fd_table[&fd].is_none() {
        return -1;
    }
    let new_fd = inner.alloc_fd();
    let value = Arc::clone(inner.fd_table[&fd].as_ref().unwrap());
    inner.fd_table.get_mut(&new_fd).unwrap().replace(value);
    new_fd as isize
}

pub fn sys_dup3(old: usize, new: usize) -> isize {
    let process = current_process();
    let mut inner = process.try_inner_exclusive_access().unwrap();
    if inner.fd_table[&old].is_none() {
        return -1;
    }
    if inner.fd_table.contains_key(&new) {
        return -1;
    }
    let value = Arc::clone(inner.fd_table[&old].as_ref().unwrap());
    inner.fd_table.insert(new, Some(value));
    new as isize
}

pub fn sys_getcwd(buf: *const u8, len: usize) -> isize {
    let token = current_user_token();
    let process = current_process();
    let inner = process.try_inner_exclusive_access().unwrap();
    let name = inner.dir_entry.as_ref().unwrap().name();
    let dir = name.as_bytes();
    for b in UserBuffer::new(translated_byte_buffer(token, buf, len)).buffers {
        b[0..dir.len()].copy_from_slice(dir);
    }
    1
}

pub fn sys_fstat(fd: isize, ptr: *mut Kstat) -> isize {
    let token = current_user_token();
    let stat = translated_refmut(token, ptr);
    let process = current_process();
    let inner = process.try_inner_exclusive_access().unwrap();
    if let Some(opt) = inner.fd_table.get(&(fd as usize)) {
        if let Some(file) = opt {
            file.kstat(stat);
            return 0;
        }
    }
    1
}

pub fn sys_getdents(fd: isize, ptr: *mut Dirent, _len: usize) -> isize {
    let token = current_user_token();
    let process = current_process();
    let inner = process.try_inner_exclusive_access().unwrap();
    let dirent = translated_refmut(token, ptr);
    let mut name = translated_str(token, (ptr as usize + 19) as *const u8);
    match inner.fd_table.get(&(fd as usize)) {
        Some(Some(file)) => {
            file.getdents(dirent);
            name.clone_from(&file.name());
            1
        }
        Some(None) => -1,
        None => -1,
    }
}

pub fn _link() -> isize {
    1
}

pub fn sys_mmap(
    start: usize,
    len: usize,
    _port: usize,
    _flag: usize,
    fd: usize,
    off: usize,
) -> isize {
    let token = current_user_token();
    let process = current_process();
    let inner = process.try_inner_exclusive_access().unwrap();
    if let Some(opt) = inner.fd_table.get(&fd) {
        if let Some(file) = opt {
            let start_addr = if start == 0 {
                sys_brk(0) as usize
            } else {
                start
            };
            sys_brk(start_addr + len);
            file.seek(SeekFrom::Start(off as u64));
            file.read(UserBuffer::new(translated_byte_buffer(
                token,
                start_addr as *const u8,
                len,
            )));
            return start_addr as isize;
        }
    }
    -1
}

pub fn sys_munmap(start: usize, _len: usize) -> isize {
    sys_brk(start);
    0
}

pub fn sys_mount() -> isize {
    0
}
pub fn sys_umount() -> isize {
    0
}
