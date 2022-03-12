use crate::sync::UPSafeCell;

use super::UartDevice;
use alloc::{collections::VecDeque, sync::Arc};

lazy_static::lazy_static! {
    pub static ref UART_DEVICE: Arc<dyn UartDevice> = Arc::new(UartHs::new());
}

pub struct UartHs {
    #[allow(unused)]
    buffer: UPSafeCell<VecDeque<u8>>,
}

impl UartHs {
    pub fn new() -> Self {
        Self {
            buffer: unsafe{ UPSafeCell::new(VecDeque::new()) },
        }
    }
}

impl UartDevice for UartHs {
    fn getchar(&self) -> Option<u8> {
        self.buffer.exclusive_access().pop_front()
    }

    fn putchar(&self, ch: u8) {
        unsafe {
            let ptr = k210_pac::UARTHS::ptr();
            while (*ptr).txdata.read().full().bit() {
                continue;
            }
            (*ptr).txdata.write(|w| w.data().bits(ch))
        }
    }

    fn handler_interrupt(&self) {
        unsafe {
            let ptr = k210_pac::UARTHS::ptr();
            let recv = (*ptr).rxdata.read();
            match recv.empty().bits() {
                true => panic!("Not char"),
                false => { 
                    let ch = recv.data().bits();
                    self.buffer.exclusive_access().push_back(ch);
                },
            }
        }
    }
}
