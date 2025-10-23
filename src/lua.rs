use crate::console_ldd::console_write_blocking;
use core::ffi::{c_char, c_int, c_long, c_void};
use defmt::*;

unsafe extern "C" {
    pub unsafe fn luaL_newstate() -> *mut c_void;
    pub unsafe fn luaL_openlibs(state: *mut c_void);
    pub unsafe fn lua_tolstring(
        state: *mut c_void,
        index: c_int,
        len: *mut c_long,
    ) -> *const c_char;
    pub unsafe fn luaL_error(state: *mut c_void, fmt: *const c_char, ...) -> c_int;

    pub unsafe fn lua_pushcclosure(
        state: *mut c_void,
        f: unsafe extern "C-unwind" fn(state: *mut c_void) -> c_int,
        n: c_int,
    );

    pub unsafe fn lua_pcallk(
        state: *mut c_void,
        nargs: c_int,
        nresults: c_int,
        errfunc: c_int,
        ctx: isize,
        k: *const c_void,
    ) -> c_int;

    pub unsafe fn lua_getglobal(state: *mut c_void, k: *const c_char) -> c_int;

    pub unsafe fn lua_settop(state: *mut c_void, idx: c_int);
    //pub unsafe fn luaL_loadbufferx(
    //    state: *mut c_void,
    //    buff: *const c_char,
    //    sz: usize,
    //    name: *const c_char,
    //    mode: *const c_char,
    //) -> c_int;
    pub unsafe fn luaL_loadstring(state: *mut c_void, s: *const c_char) -> c_int;
    pub unsafe fn lua_close(state: *mut c_void);
    pub unsafe fn lua_tointegerx(state: *mut c_void, idx: c_int, isnum: *mut c_int) -> c_int;
}

const LUA_OK: i32 = 0;
//const LUA_YIELD: i32 = 1;
const LUA_ERRRUN: i32 = 2;
//const LUA_ERRSYNTAX: i32 = 3;
//const LUA_ERRMEM: i32 = 4;
//const LUA_ERRERR: i32 = 5;

//const LUA_MULTRET: i32 = -1;

macro_rules! my_assert {
    ($condition:expr) => {
        if !$condition {
            crate::panic!("Assertion failed: {}", stringify!($condition));
        }
    };
}

pub unsafe fn lua_pcall(
    state: *mut c_void,
    nargs: c_int,
    nresults: c_int,
    errfunc: c_int,
) -> c_int {
    unsafe { lua_pcallk(state, nargs, nresults, errfunc, 0, core::ptr::null()) }
}

// print a string
#[unsafe(no_mangle)]
pub extern "C" fn lua_writestring(s: *const c_char, l: usize) {
    //fwrite((s), sizeof(char), (l), stdout)
    if !s.is_null() {
        let byte_slice: &[u8] = unsafe { core::slice::from_raw_parts(s, l) };
        unsafe {
            console_write_blocking(str::from_utf8_unchecked(byte_slice)).unwrap();
        }
    }
}

// print a newline and flush the output
#[unsafe(no_mangle)]
pub extern "C" fn lua_writeline() {
    //lua_writestring("\n", 1)
    // fflush(stdout)
    console_write_blocking("\n").unwrap();
}

/* print an error message */
#[unsafe(no_mangle)]
pub extern "C" fn lua_writestringerror(s: *const c_char, l: usize) {
    //fprintf(stderr, (s), (l))
    //fflush(stderr)
    if !s.is_null() {
        let byte_slice: &[u8] = unsafe { core::slice::from_raw_parts(s, l) };
        unsafe {
            console_write_blocking(str::from_utf8_unchecked(byte_slice)).unwrap();
        }
    }
}

unsafe fn to_string<'a>(state: *mut c_void, index: c_int) -> &'a str {
    let mut len: c_long = 0;
    unsafe {
        let ptr = lua_tolstring(state, index, &mut len);
        let bytes = core::slice::from_raw_parts(ptr, len as usize);
        core::str::from_utf8(bytes).unwrap()
    }
}

fn to_cstring(rstring: &str, buffer: &mut [u8; 256]) -> *const c_char {
    let rstring_as_bytes = rstring.as_bytes();
    let rstring_as_bytes_len = rstring_as_bytes.len();
    buffer[..rstring_as_bytes_len].copy_from_slice(rstring_as_bytes);
    buffer[rstring_as_bytes_len] = 0; // Add null terminator
    buffer.as_ptr()
}

unsafe fn report(state: *mut c_void, status: i32) -> i32 {
    if status != LUA_OK {
        unsafe {
            let mut len: c_long = 0;
            let msg = lua_tolstring(state, -1, &mut len);
            lua_writestring(msg, len as usize);
            lua_writeline();
            lua_settop(state, -(1) - 1) // lua_pop(state, 1)
        }
    }
    status
}

unsafe fn dostring(state: *mut c_void, script: &str, nret: i32) -> i32 {
    let mut buffer: [u8; 256] = [0; 256];
    let script_as_cstring = to_cstring(script, &mut buffer);

    unsafe {
        let mut status = luaL_loadstring(state, script_as_cstring);
        if status == LUA_OK {
            status = lua_pcall(state, 0, nret, 0);
        }
        report(state, status)
    }
}

unsafe fn test_version(state: *mut c_void) {
    unsafe {
        lua_getglobal(state, c"_VERSION".as_ptr());
        let version = to_string(state, -1);
        my_assert!(version == "Lua 5.4");
        info!("{}", version);
    }
}

unsafe fn test_exception(state: *mut c_void) {
    unsafe extern "C-unwind" fn it_panics(state: *mut c_void) -> c_int {
        unsafe { luaL_error(state, c"exception!".as_ptr()) }
    }

    unsafe {
        lua_pushcclosure(state, it_panics, 0);
        let result = lua_pcall(state, 0, 0, 0);
        my_assert!(result == LUA_ERRRUN);
        my_assert!(to_string(state, -1) == "exception!");
    }
}

unsafe fn test_print(state: *mut c_void) {
    unsafe {
        let script = "print(\"Hello World\")";
        let rv = dostring(state, script, 0);
        my_assert!(rv == LUA_OK);

        let script = r#"
            x = 3
            print(x)
        "#;
        let rv = dostring(state, script, 0);
        my_assert!(rv == LUA_OK);

        let script = r#"
            function fact (n)
              if n == 0 then
                return 1
              else
                return n * fact(n-1)
              end
            end
            return fact(5)
            -- print(\"5! =\", fact(5))"
        "#;
        let rv = dostring(state, script, 1);
        my_assert!(rv == LUA_OK);
        let mut isnum: c_int = 0;
        let fact_result = lua_tointegerx(state, -1, &mut isnum);
        my_assert!(isnum != 0);
        my_assert!(fact_result == 120);
        lua_settop(state, -(1) - 1); // pop fact_result off of the stack
    }
}

unsafe fn test_read(state: *mut c_void) {
    unsafe {
        let script = r#"
            x = io.read(1)
            print(x)
        "#;
        let rv = dostring(state, script, 0);
        my_assert!(rv == LUA_OK);
    }
}

pub fn test_lua() {
    unsafe {
        let state = luaL_newstate();
        my_assert!(!state.is_null());

        luaL_openlibs(state);

        test_version(state);
        test_exception(state);
        test_print(state);
        test_read(state);

        lua_close(state);
    }
}
