//! Logical device driver for the system console.

use embassy_rp::uart::{self, Async, UartRx, UartTx};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex;
use static_cell::StaticCell;

type AsyncMutex<T> = mutex::Mutex<CriticalSectionRawMutex, T>;
type UartTxAsyncMutex = AsyncMutex<UartTx<'static, uart::Async>>;
type UartRxAsyncMutex = AsyncMutex<UartRx<'static, uart::Async>>;

struct Console {
    tx_mutex_cell: StaticCell<UartTxAsyncMutex>,
    rx_mutex_cell: StaticCell<UartRxAsyncMutex>,

    // stored pointers (set once at init)
    tx_mutex_ptr: *const UartTxAsyncMutex,
    rx_mutex_ptr: *const UartRxAsyncMutex,
    tx_inner_ptr: *mut UartTx<'static, Async>,
    rx_inner_ptr: *mut UartRx<'static, Async>,
}

impl Console {
    const fn new() -> Self {
        Self {
            tx_mutex_cell: StaticCell::new(),
            rx_mutex_cell: StaticCell::new(),
            tx_mutex_ptr: core::ptr::null(),
            rx_mutex_ptr: core::ptr::null(),
            tx_inner_ptr: core::ptr::null_mut(),
            rx_inner_ptr: core::ptr::null_mut(),
        }
    }

    fn tx_mutex(&self) -> &'static UartTxAsyncMutex {
        unsafe {
            if self.tx_mutex_ptr.is_null() {
                panic!("console not initialized; call console_init() first");
            }
            &*self.tx_mutex_ptr
        }
    }

    unsafe fn tx_inner_mut(&self) -> &'static mut UartTx<'static, Async> {
        unsafe {
            if self.tx_inner_ptr.is_null() {
                panic!("console not initialized; call console_init().await first");
            }
            &mut *self.tx_inner_ptr
        }
    }
}

static CONSOLE_CELL: StaticCell<Console> = StaticCell::new();
static mut CONSOLE_PTR: *const Console = core::ptr::null();

// generic helper: take a raw mutable pointer to the inner T while holding the async lock
async fn take_inner_ptr<T>(mutex_handle: &'static AsyncMutex<T>) -> *mut T {
    let mut guard = mutex_handle.lock().await;
    let ptr: *mut T = &mut *guard as *mut _;
    // guard dropped here
    ptr
}

pub async fn console_init(tx: UartTx<'static, Async>, rx: UartRx<'static, Async>) {
    // allocate the singleton and keep a raw pointer for global access
    let console = CONSOLE_CELL.init(Console::new());
    unsafe {
        CONSOLE_PTR = console as *const _;
    }

    // init per-direction mutex cells
    let tx_handle: &'static UartTxAsyncMutex =
        unsafe { (&*CONSOLE_PTR).tx_mutex_cell.init(mutex::Mutex::new(tx)) };
    let rx_handle: &'static UartRxAsyncMutex =
        unsafe { (&*CONSOLE_PTR).rx_mutex_cell.init(mutex::Mutex::new(rx)) };

    // capture inner pointers while we can await
    let tx_inner = take_inner_ptr(tx_handle).await;
    let rx_inner = take_inner_ptr(rx_handle).await;

    unsafe {
        let c = &*CONSOLE_PTR as *const Console as *mut Console;
        (*c).tx_mutex_ptr = tx_handle as *const _;
        (*c).rx_mutex_ptr = rx_handle as *const _;
        (*c).tx_inner_ptr = tx_inner;
        (*c).rx_inner_ptr = rx_inner;
    }
}

fn console() -> &'static Console {
    unsafe {
        if CONSOLE_PTR.is_null() {
            panic!("console not initialized; call console_init() first");
        }
        &*CONSOLE_PTR
    }
}

pub async fn console_write(out_string: &str) {
    let uart_mutex = console().tx_mutex();
    let mut guard = uart_mutex.lock().await;
    guard.write(out_string.as_bytes()).await.unwrap();
}

// SAFETY: Only safe if all sync and async console users are running under the same
//         Embassy executor.
pub fn console_write_blocking(out_string: &str) -> Result<(), uart::Error> {
    let tx = unsafe { console().tx_inner_mut() };
    tx.blocking_write(out_string.as_bytes())
}

pub fn console_read_blocking() -> Result<u8, uart::Error> {
    Ok(b'a')
}
