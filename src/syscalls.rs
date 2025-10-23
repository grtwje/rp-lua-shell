//! Newlib-nano has a set of 17 system calls that glue the C lib to your "OS."
//!
//! Not all 17 are implemented here. They are only implemented as needed by the
//! application.
use crate::console_ldd::{console_read_blocking, console_write_blocking};
use core::ffi::{c_char, c_int};
use defmt::panic;
use defmt::*;

static mut SBRK_HEAP: [u8; 2064] = [0; 2064];
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
            panic!(
                "_sbrk OOM: {} + {} = {} > {}",
                SBRK_HEAP_PTR,
                incr,
                SBRK_HEAP_PTR + (incr as usize),
                heap_len
            );
            //return core::ptr::null_mut();
        }
        let prev = SBRK_HEAP_PTR;
        SBRK_HEAP_PTR += incr as usize;
        SBRK_HEAP.as_mut_ptr().wrapping_add(prev)
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn _write(file: c_int, buf: *const c_char, len: c_int) -> c_int {
    if file != 1 && file != 2 && file != 3 {
        return -1;
    }

    let max_len = core::cmp::min(len, 128) as usize;
    unsafe {
        if !buf.is_null() {
            let bytes = core::slice::from_raw_parts(buf, max_len);
            match core::str::from_utf8(&bytes[..max_len]) {
                Ok(s) => {
                    if console_write_blocking(s).is_err() {
                        return -1;
                    }
                }
                Err(_) => {
                    info!("_write: Invalid UTF-8 sequence");
                    return -1;
                }
            }
        }
    }
    max_len as i32
}

#[unsafe(no_mangle)]
pub extern "C" fn _read(file: c_int, ptr: *mut c_char, len: c_int) -> c_int {
    if file != 0 {
        return -1;
    }

    let mut out_ptr = ptr;
    for _ in 0..len {
        match console_read_blocking() {
            Ok(c) => unsafe {
                *out_ptr = c as c_char;
                out_ptr = out_ptr.add(size_of::<c_char>())
            },
            Err(_) => {
                info!("_read: console read error");
                return -1;
            }
        }
    }

    len
}
