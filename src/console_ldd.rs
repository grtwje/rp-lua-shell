//! Logical device driver for the system console.

use embassy_rp::uart::{self, Async, UartRx, UartTx};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex;
use static_cell::StaticCell;

type UartAsyncMutex = mutex::Mutex<CriticalSectionRawMutex, UartTx<'static, uart::Async>>;

static UART_TX: StaticCell<UartAsyncMutex> = StaticCell::new();
static mut GLOBAL_UART_PTR: *const UartAsyncMutex = core::ptr::null();
static mut GLOBAL_TX_PTR: *mut UartTx<'static, Async> = core::ptr::null_mut();

pub async fn console_init(tx: UartTx<'static, Async>, _rx: UartRx<'static, Async>) {
    // initialize the static mutex that will be used by async callers
    let handle: &'static UartAsyncMutex = UART_TX.init(mutex::Mutex::new(tx));

    // obtain a raw pointer to the inner UartTx while we can await here
    // SAFETY: we're holding the mutex guard while taking the pointer, then drop the guard.
    {
        let mut guard = handle.lock().await; // async lock here only at init time

        // take a raw mutable pointer to the inner UartTx while we hold the lock
        let tx_ptr: *mut UartTx<'static, Async> = &mut *guard as *mut _;
        unsafe {
            GLOBAL_TX_PTR = tx_ptr;
        }
        // guard is dropped here
    }

    // store pointer to the mutex itself for console_write (async path)
    unsafe {
        GLOBAL_UART_PTR = handle as *const _;
    }
}

pub async fn console_write(out_string: &str) {
    // SAFETY: console_init must be called once at startup before any callers run.
    let uart: &UartAsyncMutex = unsafe {
        if GLOBAL_UART_PTR.is_null() {
            panic!("console not initialized; call console_init() before using console_write");
        }
        &*GLOBAL_UART_PTR
    };

    let mut guard = uart.lock().await;
    guard.write(out_string.as_bytes()).await.unwrap();
}

pub fn console_write_blocking(out_string: &str) {
    // SAFETY: console_init must be called once (and awaited) at startup before any blocking callers.
    let tx: &mut UartTx<'static, Async> = unsafe {
        if GLOBAL_TX_PTR.is_null() {
            panic!(
                "console not initialized; call console_init().await before using console_write_blocking"
            );
        }
        &mut *GLOBAL_TX_PTR
    };

    // Use the blocking write method on the UartTx (this is unsafe-ish but OK per caller guarantee).
    // If your UartTx provides `blocking_write`, call it; otherwise replace with your HAL blocking API.
    tx.blocking_write(out_string.as_bytes()).unwrap();
}
