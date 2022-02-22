use super::io::{IoBase, Read, Seek, SeekFrom, Write};
use crate::drivers::BLOCK_DEVICE;
use alloc::{collections::VecDeque, sync::Arc};
use core::cmp::min;
use core::convert::TryFrom;
use crate::drivers::BlockDevice;
use spin::mutex::Mutex;
#[derive(Debug)]
pub struct BlockCache {
    pub pos: usize,
    pub block_id: usize,
    pub cache: [u8; 512],
}

impl BlockCache {
    fn new(p: usize, block_drv: Arc<dyn BlockDevice>) -> Self {
        let mut cache = [0; 512];
        let block_id = p / 512;
        let pos = p % 512;
        block_drv.read_block(block_id, &mut cache);
        Self {
            pos,
            block_id,
            cache,
        }
    }
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
    pub block_driver: Arc<dyn BlockDevice>,
    #[allow(unused)]
    pub queue: VecDeque<(usize, Arc<Mutex<BlockCache>>)>,
}

impl BlkCacheManager {
    pub fn new() -> Self {
        BlkCacheManager {
            pos: 0,
            block_driver: BLOCK_DEVICE.clone(),
            queue: VecDeque::new(),
        }
    }
}

impl Iterator for BlkCacheManager {
    type Item = Arc<Mutex<BlockCache>>;
    fn next(&mut self) -> Option<Self::Item> {
        let blk_cache = BlockCache::new(self.pos, self.block_driver.clone());
        Some(Arc::new(Mutex::new(blk_cache)))
    }
}

impl IoBase for BlkCacheManager {
    type Error = ();
}

impl Read for BlkCacheManager {
    fn read(&mut self, mut buf: &mut [u8]) -> Result<usize, Self::Error> {
        let mut len = 0;
        while !buf.is_empty() {
            let blk_lock = self.next().unwrap();
            let mut blk = blk_lock.lock();
            let n = blk.read(buf).unwrap();
            let tmp = buf;
            buf = &mut tmp[n..];
            self.pos += n;
            len += n;
        }
        Ok(len)
    }
}

impl Write for BlkCacheManager {
    fn write(&mut self, mut buf: &[u8]) -> Result<usize, Self::Error> {
        let len = buf.len();
        while !buf.is_empty() {
            let blk_lock = self.next().unwrap();
            let mut blk = blk_lock.lock();
            let n = blk.write(buf)?;
            let tmp = buf;
            buf = &tmp[n..];
            self.pos += n;
        }
        Ok(len)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl Seek for BlkCacheManager {
    fn seek(&mut self, pos: super::io::SeekFrom) -> Result<u64, Self::Error> {
        let new_offset_opt: Option<u32> = match pos {
            SeekFrom::Start(x) => u32::try_from(x).ok(),
            SeekFrom::Current(x) => u32::try_from(self.pos + x as usize).ok(),
            SeekFrom::End(_) => panic!("Seek Error"),
        };
        self.pos = new_offset_opt.unwrap() as usize;
        Ok(0)
    }
}
