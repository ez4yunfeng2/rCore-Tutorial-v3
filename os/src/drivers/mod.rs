mod block;
mod uart;
mod plic;
use core::any::Any;
use k210_pac::Interrupt;

pub use block::BLOCK_DEVICE;
pub use plic::PLIC_DRIVE;
pub use uart::UART_DEVICE;
pub trait BlockDevice: Send + Sync + Any {
    fn read_block(&self, block_id: usize, buf: &mut [u8]);
    fn write_block(&self, block_id: usize, buf: &[u8]);
    fn change_mode(&self) {}
    fn handler_interrupt(&self) {}
}

pub trait UartDevice: Send + Sync + Any {
    fn getchar(&self) -> Option<u8>;
    fn putchar(&self, ch: u8);
    fn handler_interrupt(&self);
}

pub trait PlicDevice: Send + Sync + Any {
    fn enable(&self,source: Interrupt, hartid: usize);
    fn current(&self, hartid: usize) -> usize;
    fn clear(&self, irq: usize, hartid: usize);
    fn set_priority(&self, value: u32, pin: Interrupt);
    fn set_thershold(&self, value: u32, hartid: usize);
}