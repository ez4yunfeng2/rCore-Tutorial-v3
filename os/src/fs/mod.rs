mod inode;
mod pipe;
mod stdio;

use crate::{fatfs::io::SeekFrom, mm::UserBuffer};

pub trait File: Send + Sync {
    fn readable(&self) -> bool;
    fn writable(&self) -> bool;
    fn seek(&self, _offset: SeekFrom) -> usize {0}
    fn read(&self, _buf: UserBuffer) -> usize {0}
    fn write(&self, _buf: UserBuffer) -> usize {0}
    fn create(&self, _name: &str, _read: bool, _write: bool, _isdir: bool) -> Option<Arc<OSInode>> {None}
    fn open(&self, _name: &str, _read: bool, _write: bool, _isdir:bool) -> Option<Arc<OSInode>> {None}
    fn remove(&self,_path:&str) -> bool { false }
    fn islink(&self) -> bool { false }
    fn kstat(&self,_stat:&mut Kstat) {}
    fn name(&self) -> String;
}

use alloc::{string::String, sync::Arc};
pub use inode::{open_file, root, OSInode, OpenFlags,Kstat,Dirent};
pub use pipe::{make_pipe, Pipe};
pub use stdio::{Stdin, Stdout};
