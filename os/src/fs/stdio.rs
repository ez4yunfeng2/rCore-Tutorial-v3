use super::File;
use crate::drivers::UART_DEVICE;
use crate::irq::wait_for_irq_and_run_next;
use crate::mm::UserBuffer;
use crate::sbi::console_getchar;
use alloc::string::String;
use k210_pac::Interrupt;
use riscv::register::sstatus;
pub struct Stdin;

pub struct Stdout;

impl File for Stdin {
    fn readable(&self) -> bool {
        true
    }
    fn writable(&self) -> bool {
        false
    }
    fn read(&self, mut user_buf: UserBuffer) -> usize {
        // println!("Read Char");
        assert_eq!(user_buf.len(), 1);
        let mut ch;
        loop {
            match UART_DEVICE.getchar() {
                Some(c) => {
                    ch = c;
                    break;
                }
                None => wait_for_irq_and_run_next(Interrupt::UARTHS as usize),
            };
        }
        // loop {
        //     ch = console_getchar();
        //     if ch != 0 {
        //         break;
        //     }
        // }
        unsafe {
            user_buf.buffers[0]
                .as_mut_ptr()
                .write_volatile((ch & 0xff) as u8);
        }
        1
    }

    fn name(&self) -> String {
        String::from("Stdin")
    }
}

impl File for Stdout {
    fn readable(&self) -> bool {
        false
    }
    fn writable(&self) -> bool {
        true
    }

    fn write(&self, user_buf: UserBuffer) -> usize {
        for buffer in user_buf.buffers.iter() {
            print!("{}", core::str::from_utf8(*buffer).unwrap());
        }
        user_buf.len()
    }

    fn name(&self) -> String {
        String::from("Stdout")
    }
}
