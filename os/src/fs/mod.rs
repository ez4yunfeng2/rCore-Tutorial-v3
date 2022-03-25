#![allow(unused)]
mod inode;
mod pipe;
mod stdio;

use crate::{fatfs::io::SeekFrom, mm::UserBuffer};

pub trait File: Send + Sync {
    fn readable(&self) -> bool;
    fn writable(&self) -> bool;
    fn seek(&self, _offset: SeekFrom) -> usize {
        0
    }
    fn read(&self, buf: UserBuffer) -> usize {
        0
    }
    fn write(&self, buf: UserBuffer) -> usize {
        0
    }
    fn create(&self, name: &str, read: bool, write: bool, isdir: bool) -> Option<Arc<OSInode>> {
        None
    }
    fn open(&self, name: &str, read: bool, write: bool, isdir: bool) -> Option<Arc<OSInode>> {
        None
    }
    fn remove(&self, path: &str) -> bool {
        false
    }
    fn islink(&self) -> bool {
        false
    }
    fn kstat(&self, stat: &mut Kstat) {}
    fn name(&self) -> String;
    fn getdents(&self, dirent: &mut Dirent) -> isize {
        -1
    }
}

use alloc::{string::String, sync::Arc};
pub use inode::{open_file, root, Dirent, Kstat, OSInode, OpenFlags};
pub use pipe::{make_pipe, Pipe};
pub use stdio::{Stdin, Stdout};
