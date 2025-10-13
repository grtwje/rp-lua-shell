use core::alloc::{GlobalAlloc, Layout};
use core::ffi::c_void;
use defmt::*;

#[global_allocator]
static ALLOCATOR: emballoc::Allocator<16384> = emballoc::Allocator::new();
extern crate alloc;

// This will be called instead of malloc
#[unsafe(no_mangle)]
pub extern "C" fn malloc(size: usize) -> *mut c_void {
    unsafe {
        let cookie_size = core::mem::size_of::<usize>();
        let layout = Layout::from_size_align(size + cookie_size, 1).unwrap();
        let ptr = ALLOCATOR.alloc(layout) as *mut c_void;
        if ptr.is_null() {
            info!("malloc {} OOM", size);
            return core::ptr::null_mut();
        }
        *(ptr as *mut usize) = size;
        ptr.add(cookie_size)
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn realloc(ptr: *mut c_void, new_size: usize) -> *mut c_void {
    unsafe {
        if new_size == 0 {
            if !ptr.is_null() {
                free(ptr);
            }
            return core::ptr::null_mut(); // Equivalent to NULL in C
        }

        if ptr.is_null() {
            // realloc(NULL, size) is equivalent to malloc(size)
            return malloc(new_size);
        }

        // Get the original pointer and size
        let cookie_size = core::mem::size_of::<usize>();
        let real_ptr = (ptr as *mut u8).sub(cookie_size);
        let old_size = *(real_ptr as *const usize);

        // Allocate new block
        let layout = Layout::from_size_align(new_size + cookie_size, 1).unwrap();
        let new_ptr = ALLOCATOR.alloc(layout);
        if new_ptr.is_null() {
            info!("realloc {} OOM", new_size);
            return core::ptr::null_mut();
        }

        // Store new size
        *(new_ptr as *mut usize) = new_size;

        // Copy old data (up to min(old_size, new_size))
        let copy_size = core::cmp::min(old_size, new_size);
        core::ptr::copy_nonoverlapping(ptr as *const u8, new_ptr.add(cookie_size), copy_size);

        // Free old block
        let old_layout = Layout::from_size_align(old_size + cookie_size, 1).unwrap();
        ALLOCATOR.dealloc(real_ptr, old_layout);

        // Return pointer to user data
        new_ptr.add(cookie_size) as *mut c_void
    }
}

// This will be called instead of free
#[unsafe(no_mangle)]
#[inline(never)]
pub extern "C" fn free(ptr: *mut c_void) {
    unsafe {
        if !ptr.is_null() {
            let cookie_size = core::mem::size_of::<usize>();
            let real_ptr = (ptr as *mut u8).sub(cookie_size);
            let size = *(real_ptr as *const usize);
            let layout = Layout::from_size_align(size + cookie_size, 1).unwrap();
            ALLOCATOR.dealloc(real_ptr, layout);
        }
    }
}
