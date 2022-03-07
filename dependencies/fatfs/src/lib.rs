#![allow(unused)]
#![no_std]
pub mod boot_sector;
pub mod dir_entry;
pub mod file;
pub mod fs;
pub mod io;
pub mod lfn;
pub mod sdcard;
pub mod table;
pub mod time;
extern crate alloc;
use alloc::sync::Arc;
use dir_entry::DirEntry;
use file::Inode;
use io::Error;
use lazy_static::lazy_static;

use crate::{fs::FileSystem, sdcard::BlkCacheManager};
lazy_static! {
    pub static ref FATFS: Arc<FileSystem<BlkCacheManager>> =
        Arc::new(FileSystem::new(BlkCacheManager::new()).unwrap());
}

pub fn fs_init() {
    root_dir().ls()
}

#[inline]
pub fn alloc_cluster(prev_cluster: Option<u32>, zero: bool) -> Result<u32, Error<()>> {
    FATFS.alloc_cluster(prev_cluster, zero)
}

#[inline]
pub fn root_dir() -> Inode {
    Inode::Dir(DirEntry::root_dir(FATFS.bpb.root_dir_first_cluster))
}

#[inline]
pub fn cluster_to_offset(cluster: u32) -> u64 {
    FATFS.byte_offset(cluster) as u64
}
