use alloc::sync::Arc;
use log::{LevelFilter, Log, Metadata, Record, Level};
use lazy_static::lazy_static;
use spin::mutex::Mutex;
use crate::{drivers::{UART_DEVICE, UartDevice}};
use core::fmt::{self, Write};

struct Stdout(Arc<dyn UartDevice>);

lazy_static!(
    static ref STDOUT: Mutex<Stdout> = Mutex::new(Stdout(UART_DEVICE.clone()));
);

impl Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            self.0.putchar(c as u8);
            // UART_DEVICE.putchar(c as u8);
            // use crate::sbi::console_putchar;
            // console_putchar(c as usize)
        }
        Ok(())
    }
}

pub fn print(args: fmt::Arguments) {
    STDOUT.lock().write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!($fmt $(, $($arg)+)?))
    }
}

#[macro_export]
macro_rules! println {
    ($fmt: literal $(, $($arg: tt)+)?) => {
        $crate::console::print(format_args!(concat!($fmt, "\n") $(, $($arg)+)?))
    }
}

#[allow(unused)]
pub fn logger_init() {
    static LOGGER: SimpleLogger = SimpleLogger;
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(LevelFilter::Info);
}

macro_rules! with_color {
    ($args: ident, $color_code: ident) => {{
        format_args!("\u{1B}[{}m{}\u{1B}[0m", $color_code as u8, $args)
    }};
}

fn print_in_color(args: fmt::Arguments, color_code: u8) {
    print(with_color!(args, color_code))
}

struct SimpleLogger;

impl Log for SimpleLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }
    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        print_in_color(
            format_args!("[{:>5}] {}\n", record.level(), record.args()),
            level_to_color_code(record.level()),
        );
    }
    fn flush(&self) {}
}

fn level_to_color_code(level: Level) -> u8 {
    match level {
        Level::Error => 31, // Red
        Level::Warn => 93,  // BrightYellow
        Level::Info => 34,  // Blue
        Level::Debug => 32, // Green
        Level::Trace => 90, // BrightBlack
    }
}