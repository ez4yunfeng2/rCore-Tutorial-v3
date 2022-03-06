use super::UartDevice;
use alloc::{collections::VecDeque, sync::Arc};

lazy_static::lazy_static! {
    pub static ref UART_DEVICE: Arc<dyn UartDevice> = Arc::new(UartHs::new());
}

pub struct UartHs {
    #[allow(unused)]
    buffer: VecDeque<u8>,
}

impl UartHs {
    pub fn new() -> Self {
        Self {
            buffer: VecDeque::new(),
        }
    }
}

impl UartDevice for UartHs {
    fn getchar(&self) -> Option<u8> {
        unsafe {
            let ptr = k210_pac::UARTHS::ptr();
            let recv = (*ptr).rxdata.read();
            match recv.empty().bits() {
                true => None,
                false => Some(recv.data().bits() & 0xff),
            }
        }
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
        todo!()
    }
}
