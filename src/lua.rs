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
    pub unsafe fn lua_getglobal(state: *mut c_void, k: *const c_char) -> c_int;
}

macro_rules! my_assert {
    ($condition:expr) => {
        if !$condition {
            crate::panic!("Assertion failed: {}", stringify!($condition));
        }
    };
}

unsafe fn to_string<'a>(state: *mut c_void, index: c_int) -> &'a str {
    let mut len: c_long = 0;
    unsafe {
        let ptr = lua_tolstring(state, index, &mut len);
        let bytes = core::slice::from_raw_parts(ptr, len as usize);
        core::str::from_utf8(bytes).unwrap()
    }
}

pub fn test_lua() {
    unsafe {
        let state = luaL_newstate();
        my_assert!(!state.is_null());

        luaL_openlibs(state);

        lua_getglobal(state, c"_VERSION".as_ptr());
        let version = to_string(state, -1);
        my_assert!(version == "Lua 5.4");

        info!("{}", version);
    }
}
