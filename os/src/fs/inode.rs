use crate::fatfs::file::Inode;
use crate::fatfs::io::SeekFrom;
use crate::fatfs::root_dir;
use crate::fs::File;
use crate::mm::UserBuffer;
use crate::sync::UPSafeCell;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;

pub struct OSInode {
    readable: bool,
    writable: bool,
    inner: UPSafeCell<OSInodeInner>,
}
pub struct OSInodeInner {
    offset: usize,
    inode: Arc<UPSafeCell<Inode>>,
}

impl OSInode {
    pub fn new(readable: bool, writable: bool, inode: Arc<UPSafeCell<Inode>>) -> Self {
        Self {
            readable,
            writable,
            inner: unsafe { UPSafeCell::new(OSInodeInner { offset: 0, inode }) },
        }
    }

    pub fn read_all(&self) -> Vec<u8> {
        let inner = self.inner.exclusive_access();
        let offset = inner.offset;
        let v = inner.inode.inner.borrow_mut().read_all(offset);
        v
    }
}

bitflags! {
    pub struct OpenFlags: u32 {
        const RDONLY = 0x000;
        const WRONLY = 0x001;
        const RDWR = 0x002;
        const CREATE = 0x40;
        const TRUNC = 1 << 10;
        const DIRECTORY = 0x0200000;
        const DIR = 0x040000;
        const FILE = 0x100000;
    }
}

impl OpenFlags {
    /// Do not check validity for simplicity
    /// Return (readable, writable)
    pub fn read_write(&self) -> (bool, bool) {
        if self.is_empty() {
            (true, false)
        } else if self.contains(Self::WRONLY) {
            (false, true)
        } else {
            (true, true)
        }
    }
}

pub fn root() -> Arc<OSInode> {
    Arc::new(OSInode::new(
        true,
        true,
        Arc::new(unsafe { UPSafeCell::new(root_dir()) }),
    ))
}

pub fn open_file(path: &str, flags: OpenFlags) -> Option<Arc<OSInode>> {
    println!("Open: {}", path);
    let (readable, writable) = flags.read_write();
    if flags.contains(OpenFlags::CREATE) {
        let file = root_dir().create(path, false).unwrap();
        Some(Arc::new(OSInode::new(
            readable,
            writable,
            Arc::new(unsafe { UPSafeCell::new(file) }),
        )))
    } else {
        if let Some(file) = root_dir().open(path, false) {
            Some(Arc::new(OSInode::new(
                readable,
                writable,
                Arc::new(unsafe { UPSafeCell::new(file) }),
            )))
        } else {
            None
        }
    }
}

impl File for OSInode {
    fn readable(&self) -> bool {
        self.readable
    }
    fn writable(&self) -> bool {
        self.writable
    }
    
    fn open(&self, name: &str, read: bool, write: bool, isdir: bool) -> Option<Arc<OSInode>> {
        let inner = self.inner.exclusive_access();
        let mut inner = inner.inode.exclusive_access();
        if let Some(inode) = inner.open(name, isdir) {
            let os_inode = OSInode::new(read, write, Arc::new(unsafe { UPSafeCell::new(inode) }));
            Some(Arc::new(os_inode))
        } else {
            None
        }
    }

        
    fn seek(&self, offset: SeekFrom) -> usize {
        let mut inner = self.inner.exclusive_access();
        let offset = inner.inode
            .exclusive_access()
            .seek(offset);
        inner.offset = offset;
        offset
    }


    fn read(&self, mut buf: UserBuffer) -> usize {
        let mut inner = self.inner.exclusive_access();
        let mut total_read_size = 0usize;
        for slice in buf.buffers.iter_mut() {
            let offset = inner.offset;
            let len = inner.inode.exclusive_access().read(offset, *slice);
            if len == 0 {
                break;
            }
            inner.offset += len;
            total_read_size += len;
        }
        total_read_size
    }
    fn write(&self, mut buf: UserBuffer) -> usize {
        let mut inner = self.inner.exclusive_access();

        let mut total_write_size = 0usize;
        for slice in buf.buffers.iter_mut() {
            let offset = inner.offset;
            let len = inner.inode.inner.borrow_mut().write(offset, *slice);
            if len == 0 {
                break;
            }
            inner.offset += len;
            total_write_size += len;
        }
        total_write_size
    }
    fn create(&self, name: &str, read: bool, write: bool, isdir: bool) -> Option<Arc<OSInode>> {
        let inner = self.inner.exclusive_access();
        let mut inner = inner.inode.inner.borrow_mut();
        if let Some(inode) = inner.create(name, isdir) {
            let os_inode = OSInode::new(read, write, Arc::new(unsafe { UPSafeCell::new(inode) }));
            Some(Arc::new(os_inode))
        } else {
            None
        }
    }

    fn kstat(&self, stat: &mut Kstat) {
        self.inner
            .exclusive_access()
            .inode
            .inner
            .borrow_mut()
            .stat(stat)
    }

    fn remove(&self, path: &str) -> bool {
        self.inner
            .exclusive_access()
            .inode
            .inner
            .borrow_mut()
            .remove(path)
    }

    fn name(&self) -> String {
        self.inner
            .inner
            .borrow_mut()
            .inode
            .inner
            .borrow_mut()
            .file_name()
    }
}

#[repr(C)]
pub struct Kstat {
    pub st_dev: u64,
    pub sd_ino: u64,
    pub st_mode: u32,
    pub st_nlink: u32,
    pub st_uid: u32,
    pub st_gid: u32,
    pub st_rdev: u64,
    pub __pad: u64,
    pub st_size: isize,
    pub st_blksize: u32,
    pub __pad2: i32,
    pub st_blocks: u64,
    pub st_atime_sec: u64,
    pub st_atime_nsec: u64,
    pub st_mtime_sec: u64,
    pub st_mtime_nsec: u64,
    pub st_ctime_sec: u64,
    pub st_ctime_nsec: u64,
    pub __unused: [i32; 2],
}
#[repr(C)]
pub struct Dirent {
    d_ino: usize,
    d_off: isize,
    d_reclen: u16,
    d_type: u8,
    name: *const u8,
}
