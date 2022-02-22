#![allow(non_camel_case_types)]

use super::utils::set_bit;
use k210_pac as pac;
#[derive(Copy, Clone, PartialEq, Eq)]
pub enum interrupt {
    NO = 0,
    SPI0,
    SPI1,
    SPI_SLAVE,
    SPI3,
    I2S0,
    I2S1,
    I2S2,
    I2C0,
    I2C1,
    I2C2,
    UART1,
    UART2,
    UART3,
    TIMER0A,
    TIMER0B,
    TIMER1A,
    TIMER1B,
    ITMER2A,
    TIMER2B,
    RTC,
    WDT0,
    WDT1,
    APB,
    DVP,
    AL,
    FFT,
    DMA0,
    DMA1,
    DMA2,
    DMA3,
    DMA4,
    DMA5,
    UARTHS,
    GPIOHS0,
    GPIOHS1,
    GPIOHS2,
    GPIOHS3,
    GPIOHS4,
    GPIOHS5,
    GPIOHS6,
    GPIOHS7,
    GPIOHS8,
    GPIOHS9,
    GPIOHS10,
    GPIOHS11,
    GPIOHS12,
    GPIOHS13,
    GPIOHS14,
    GPIOHS15,
    GPIOHS16,
    GPIOHS17,
    GPIOHS18,
    GPIOHS19,
    GPIOHS20,
    GPIOHS21,
    GPIOHS22,
    GPIOHS23,
    GPIOHS24,
    GPIOHS25,
    GPIOHS26,
    GPIOHS27,
    GPIOHS28,
    GPIOHS29,
    GPIOHS30,
    GPIOHS31,
}
pub fn enable(source: interrupt) {
    unsafe {
        let idx = source as usize;
        let ptr = pac::PLIC::ptr();
        (*ptr).target_enables[0].enable[idx / 32]
            .modify(|r, w| w.bits(set_bit(r.bits(), idx as u8 % 32, true)));
    }
}

pub fn set_priority(pin: interrupt, value: u32) {
    unsafe {
        let ptr = pac::PLIC::ptr();
        (*ptr).priority[pin as usize].write(|w| w.bits(value))
    }
}

pub fn set_thershold(value: u32) {
    unsafe {
        let ptr = pac::PLIC::ptr();
        (*ptr).targets[0].threshold.write(|w| w.bits(value));
    }
}

pub fn clear_irq() {
    unsafe {
        let ptr = pac::PLIC::ptr();
        (*ptr).targets[0].claim.modify(|r,w|w.bits(r.bits()))
    }
}