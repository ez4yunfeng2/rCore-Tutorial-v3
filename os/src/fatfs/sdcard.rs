use super::io::{IoBase, Read, Seek, SeekFrom, Write};
use crate::drivers::BlockDevice;
use crate::drivers::BLOCK_DEVICE;
use crate::sync::UPSafeCell;
use alloc::collections::BTreeMap;
use alloc::{collections::VecDeque, sync::Arc};
use core::cmp::{self, min};
use core::convert::TryFrom;
use k210_pac::dmac::id;
use k210_pac::gpiohs::fall_ie;

lazy_static::lazy_static!(
    pub static ref BLK_MANAGER: Arc<UPSafeCell<BlkManager>> = Arc::new(unsafe{ UPSafeCell::new(BlkManager::new()) });
);

#[derive(Debug)]
pub struct BlockCache {
    pub pos: usize,
    pub block_id: usize,
    pub dirty: bool,
    pub cache: [u8; 512],
}

unsafe impl Sync for BlockCache {}
unsafe impl Send for BlockCache {}

impl IoBase for BlockCache {
    type Error = ();
}

impl Read for BlockCache {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        let min = min(512 - self.pos, buf.len());
        for i in 0..min {
            buf[i] = self.cache[self.pos + i];
        }
        self.pos += min;
        Ok(min)
    }
}

impl Write for BlockCache {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        let min = min(512 - self.pos, buf.len());
        for i in 0..min {
            self.cache[self.pos + i] = buf[i];
        }
        BLOCK_DEVICE.write_block(self.block_id, &mut self.cache);
        self.pos += min;
        Ok(min)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        todo!()
    }
}

impl Seek for BlockCache {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, Self::Error> {
        match pos {
            SeekFrom::Start(x) => self.pos = x as usize,
            _ => panic!("seek error"),
        }
        Ok(1)
    }
}

pub struct BlkCacheManager {
    pub pos: usize,
    pub start: usize,
    pub size: usize,
    pub block_driver: Arc<dyn BlockDevice>,
}

impl BlkCacheManager {
    pub fn new() -> Self {
        BlkCacheManager {
            pos: 0,
            size: 0,
            start: 0,
            block_driver: BLOCK_DEVICE.clone(),
        }
    }
    pub fn from(start: usize, size: usize) -> Self {
        BlkCacheManager {
            pos: 0,
            size,
            start,
            block_driver: BLOCK_DEVICE.clone(),
        }
    }
}

impl IoBase for BlkCacheManager {
    type Error = ();
}

impl Read for BlkCacheManager {
    fn read(&mut self, mut buf: &mut [u8]) -> Result<usize, Self::Error> {
        let start_pos = self.pos;
        while !buf.is_empty() {
            let offset = self.pos % 512;
            let blk_id = self.pos / 512;
            let n = BLK_MANAGER
                .inner
                .borrow_mut()
                .read_block(blk_id, &mut buf, &|blk, buf| {
                    let len = cmp::min(buf.len(), 512 - offset);
                    for idx in 0..len {
                        buf[idx] = blk.cache[offset + idx];
                    }
                    len
                });
            let tmp = buf;
            buf = &mut tmp[n..];
            self.pos += n;
        }
        Ok(self.pos - start_pos)
    }
}

impl Write for BlkCacheManager {
    fn write(&mut self, mut buf: &[u8]) -> Result<usize, Self::Error> {
        let start_pos = self.pos;
        while !buf.is_empty() {
            let offset = self.pos % 512;
            let blk_id = self.pos / 512;
            let n = BLK_MANAGER
                .inner
                .borrow_mut()
                .write_block(blk_id, &mut buf, &|blk, buf| {
                    let len = cmp::min(buf.len(), 512 - offset);
                    for idx in 0..len {
                        blk.cache[offset + idx] = buf[idx];
                    }
                    len
                });
            let tmp = buf;
            buf = &tmp[n..];
            self.pos += n;
        }
        Ok(self.pos - start_pos)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        let start = self.start / 512;
        let end = (self.start + self.size) / 512;
        for blk in start..end {
            BLK_MANAGER.exclusive_access().sync_block(blk);
        }
        Ok(())
    }
}

impl Seek for BlkCacheManager {
    fn seek(&mut self, pos: super::io::SeekFrom) -> Result<u64, Self::Error> {
        let new_offset_opt = match pos {
            SeekFrom::Start(x) => x as u32,
            SeekFrom::Current(x) => (self.pos as i64 + x) as u32,
            SeekFrom::End(_) => panic!("Seek Error"),
        };
        self.pos = new_offset_opt as usize;
        Ok(0)
    }
}

pub struct BlkManager {
    driver: Arc<dyn BlockDevice>,
    blocks: BTreeMap<usize, BlockCache>,
}

impl BlkManager {
    pub fn new() -> Self {
        Self {
            driver: BLOCK_DEVICE.clone(),
            blocks: BTreeMap::new(),
        }
    }
    pub fn read_block_from_disk(&mut self, blk_id: usize) {
        let mut buf = [0; 512];
        self.driver.read_block(blk_id, &mut buf);
        let blk = BlockCache {
            pos: 0,
            block_id: blk_id,
            cache: buf,
            dirty: false,
        };
        self.blocks.insert(blk_id, blk);
    }
    pub fn write_block_to_disk(&mut self, blk_id: usize) {
        let mut blk = self.blocks.get_mut(&blk_id).unwrap();
        self.driver.write_block(blk_id, &mut blk.cache);
    }

    pub fn sync_block(&mut self, blk: usize) {
        match self.blocks.remove(&blk) {
            Some(_) => {}
            None => {}
        }
    }

    pub fn read_block(
        &mut self,
        blk_id: usize,
        buf: &mut [u8],
        func: &dyn Fn(&BlockCache, &mut [u8]) -> usize,
    ) -> usize {
        let opt = self.blocks.get(&blk_id);
        let blk = if let Some(blk) = opt {
            blk
        } else {
            self.read_block_from_disk(blk_id);
            self.blocks.get(&blk_id).unwrap()
        };
        func.call_once((blk, buf))
    }
    pub fn write_block(
        &mut self,
        blk_id: usize,
        buf: &[u8],
        func: &dyn Fn(&mut BlockCache, &[u8]) -> usize,
    ) -> usize {
        let opt = self.blocks.get_mut(&blk_id);
        let mut blk = if let Some(blk) = opt {
            blk
        } else {
            self.read_block_from_disk(blk_id);
            self.blocks.get_mut(&blk_id).unwrap()
        };
        let len = func.call_once((&mut blk, buf));
        self.write_block_to_disk(blk_id);
        len
    }
}
