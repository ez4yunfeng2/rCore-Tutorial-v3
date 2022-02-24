use crate::sync::UPSafeCell;
use super::BlockDevice;
use nezha_sdc::MmcHost;

pub struct SDCardWrapper(UPSafeCell<MmcHost>);
impl SDCardWrapper {
    #[allow(unused)]
    pub fn new() -> Self {
        unsafe {
            nezha_sdc::sdcard_init();
            Self(UPSafeCell::new(MmcHost::new()))
        }
    }
}

impl BlockDevice for SDCardWrapper {
    fn read_block(&self, block_id: usize, buf: &mut [u8]) {
        unsafe {
            self.0.exclusive_access().set_data(buf.as_ptr() as usize);
            self.0
                .exclusive_access()
                .read_block(block_id as u32, (buf.len() / 512) as u32)
        }
    }
    fn write_block(&self, block_id: usize, buf: &[u8]) {
        unsafe {
            self.0.exclusive_access().set_data(buf.as_ptr() as usize);
            self.0
                .exclusive_access()
                .write_block(block_id as u32, (buf.len() / 512) as u32)
        }
    }
}
