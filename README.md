# Adding LUA to an Embassy RP2040

This is a personal learning project.  There are other good packages for integrating Rust and Lua.  However, I did not find one well suited for the micro controller space. And I thought the exercise of integrating the two would be fun. :smiley: I documenting the exercise in case I have to do something similar in the future and need a reference.

## Prerequisites

1. A Rust development environment. [Howto](https://rust-lang.org/learn/get-started/).
2. You have Embassy running the blinky example on a [RP2040](https://www.raspberrypi.com/products/rp2040/) ([RP2350](https://www.raspberrypi.com/products/rp2350/) should work too). [Howto](https://embassy.dev/book/#_getting_started).
3. In step 2, make sure you got the [debug probe](https://www.raspberrypi.com/products/debug-probe/) working.
4. Install [xpack-arm-none-eabi-gcc-14.2.1-1.1-win32-x64.zip](https://github.com/xpack-dev-tools/arm-none-eabi-gcc-xpack/releases/download/v14.2.1-1.1/xpack-arm-none-eabi-gcc-14.2.1-1.1-win32-x64.zip).  I installed xPack outside of the cargo project tree and hardcoded the path in the build.rs file.  You will need to do the same.  This is icky; I know.

## Progress so far

- The first issue is that Lua is written in C. How does one compile C code in a Rust project.  Turned out to be easy using the cc crate.  [Refer](https://docs.rs/cc/latest/cc/).  cc calls the C compiler supplied by xPack. See the build.rs file for the code that compiles the Lua source.
  - To slim down the LUA runtime, I changed the loadedlibs array in linint.c to load only a few basic Lua libs.  I have not removed the code from the compiled image.
- The second issue is that Lua expects to use the C standard library too.  Research led me to Newlib-nano, a C standard library suited for embedded work. [xPack](https://github.com/xpack-dev-tools) provides a Newlib-nano that can be used on the RP2040.  Newlib-nano binds to your "OS" via 17 syscalls. I have not implemented all 17, just the ones being used by this project.  See syscalls.rs
  - In alloc.rs I created wrappers around C malloc, realloc, and free (see alloc.rs).  [emballoc](https://docs.rs/emballoc/latest/emballoc/) is used for dynamic memory management.  Emballoc will provide a Rust global allocator if I need one later.  Newlib-nano *mostly* uses these 3 calls for dynamic memory.  But it will sometimes call _sbrk too. I provide a super simple_sbrk to cover these cases. I need to figure out where these _sbrk calls are coming from, and why they don't use my malloc/realloc/free.
  - I am working on the _write syscall so that I can print from Lua code.  Currently_write just sends everything to defmtt via info! macro calls.  Need to change this to send to the RP2040 UART.
