//! Newlib-nano has a set of 17 system calls that glue the C lib to your "OS."
//!
//! Not all 17 are implemented here. They are only implemented as needed by the
//! application.
use crate::console_ldd::console_write_blocking;
use core::ffi::{c_char, c_int};
use defmt::*;

static mut SBRK_HEAP: [u8; 2048] = [0; 2048];
static mut SBRK_HEAP_PTR: usize = 0;

/// Not sure why Newlib-nano is still calling _sbrk in a few spots when malloc,
/// realloc and free have been provided in alloc.rs.  NewLib-nano is calling
/// those most of the time.
#[unsafe(no_mangle)]
#[allow(static_mut_refs)]
pub extern "C" fn _sbrk(incr: isize) -> *mut u8 {
    unsafe {
        let heap_len = SBRK_HEAP.len();
        if SBRK_HEAP_PTR + (incr as usize) > heap_len {
            info!("_sbrk OOM");
            return core::ptr::null_mut();
        }
        let prev = SBRK_HEAP_PTR;
        SBRK_HEAP_PTR += incr as usize;
        SBRK_HEAP.as_mut_ptr().wrapping_add(prev)
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn _write(_file: c_int, buf: *const c_char, len: usize) -> c_int {
    let max_len = core::cmp::min(len, 128);
    unsafe {
        if !buf.is_null() {
            let bytes = core::slice::from_raw_parts(buf, max_len);
            match core::str::from_utf8(&bytes[..max_len]) {
                Ok(s) => console_write_blocking(s),
                Err(_) => info!("_write: Invalid UTF-8 sequence"),
            }
        }
    }
    max_len as i32
}
