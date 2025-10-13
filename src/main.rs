//! This example test the RP Pico on board LED.
//!
//! It does not work with the RP Pico W board. See wifi_blinky.rs.

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::gpio;
use embassy_rp::uart;
use embassy_time::Timer;
use gpio::{Level, Output};
use {defmt_rtt as _, panic_probe as _};

mod alloc;
mod lua;
mod syscalls;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let mut led = Output::new(p.PIN_25, Level::Low);

    let config = uart::Config::default();
    let mut uart = uart::Uart::new_blocking(p.UART0, p.PIN_0, p.PIN_1, config);
    uart.blocking_write("Hello, World!\r\n".as_bytes()).unwrap();

    lua::test_lua();

    loop {
        //info!("led on!");
        led.set_high();
        Timer::after_secs(1).await;

        //info!("led off!");
        led.set_low();
        Timer::after_secs(1).await;

        //uart.blocking_write("hello there!\r\n".as_bytes()).unwrap();
    }
}
