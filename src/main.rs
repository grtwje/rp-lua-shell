//! This example test the RP Pico on board LED.
//!
//! It does not work with the RP Pico W board. See wifi_blinky.rs.

#![no_std]
#![no_main]

use console_ldd::{console_init, console_write};
use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio;
use embassy_rp::peripherals::UART0;
use embassy_rp::uart::{Config, InterruptHandler, Uart};
use embassy_time::Timer;
use gpio::{Level, Output};
use {defmt_rtt as _, panic_probe as _};

mod alloc;
mod console_ldd;
mod lua;
mod syscalls;

bind_interrupts!(struct Irqs {
    UART0_IRQ => InterruptHandler<UART0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let mut led = Output::new(p.PIN_25, Level::Low);

    let uart = Uart::new(
        p.UART0,
        p.PIN_0,
        p.PIN_1,
        Irqs,
        p.DMA_CH0,
        p.DMA_CH1,
        Config::default(),
    );
    let (tx, rx) = uart.split();
    console_init(tx, rx).await;
    console_write(concat!(
        "Embassy executor version: ",
        env!("EMBASSY_EXECUTOR_VERSION"),
        "\r\n"
    ))
    .await;

    lua::test_lua();

    loop {
        info!("led on!");
        led.set_high();
        Timer::after_secs(1).await;

        info!("led off!");
        led.set_low();
        Timer::after_secs(1).await;
    }
}
