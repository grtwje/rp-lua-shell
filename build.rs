//! This build script copies the `memory.x` file from the crate root into
//! a directory where the linker can always find it at build time.
//! For many projects this is optional, as the linker always searches the
//! project root directory -- wherever `Cargo.toml` is. However, if you
//! are using a workspace or have a more complicated build setup, this
//! build script becomes required. Additionally, by requesting that
//! Cargo re-run the build script whenever `memory.x` is changed,
//! updating `memory.x` ensures a rebuild of the application with the
//! new memory settings.

use std::env;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

fn main() {
    // Put `memory.x` in our output directory and ensure it's
    // on the linker search path.
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    File::create(out.join("memory.x"))
        .unwrap()
        .write_all(include_bytes!("memory.x"))
        .unwrap();
    println!("cargo:rustc-link-search={}", out.display());

    // By default, Cargo will re-run a build script whenever
    // any file in the project changes. By specifying `memory.x`
    // here, we ensure the build script is only re-run when
    // `memory.x` is changed.
    //println!("cargo:rerun-if-changed=memory.x");

    let arm_root_path = Path::new(
        "C:/Users/Dev/xpack-arm-none-eabi-gcc-14.2.1-1.1-win32-x64/xpack-arm-none-eabi-gcc-14.2.1-1.1",
    );
    let arm_bin_path = arm_root_path.join("bin");
    let my_gcc = arm_bin_path.join("arm-none-eabi-gcc.exe");
    let my_ar = arm_bin_path.join("arm-none-eabi-ar.exe");
    let arm_lib_path = arm_root_path.join("arm-none-eabi/lib/thumb/v6-m/nofp");
    let gcc_lib_path = arm_root_path.join("lib/gcc/arm-none-eabi/14.2.1/thumb/v6-m/nofp");

    let mut build = cc::Build::new();
    build.compiler(my_gcc);
    build.archiver(my_ar);
    let lua_src_dir = Path::new("lua-5.4.8/src/");
    if let Ok(entries) = fs::read_dir(lua_src_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("c") {
                build.file(path);
            }
        }
    }

    build
        .include("lua-5.4.8/src/")
        .define("LUA_USE_C89", None)
        .define("NDEBUG", None)
        .flags([
            "-fno-builtin-free",
            "-fno-builtin-malloc",
            "-fno-builtin-calloc",
            "-fno-builtin-realloc",
            "-mthumb",
            "-mcpu=cortex-m0",
        ])
        .compile("lua");

    // Link against Newlib
    println!("cargo:rustc-link-lib=c_nano"); // Link against the C standard library (Newlib)
    println!("cargo:rustc-link-lib=nosys"); // Link against nosys for bare-metal support
    println!(
        "cargo:rustc-link-search=native={}",
        arm_lib_path.to_string_lossy()
    ); // Path to Newlib if not in standard location

    println!(
        "cargo:rustc-link-search=native={}",
        gcc_lib_path.to_string_lossy()
    );
    println!("cargo:rustc-link-lib=gcc");

    println!("cargo:rustc-link-arg-bins=--nmagic");
    println!("cargo:rustc-link-arg-bins=-Tlink.x");
    println!("cargo:rustc-link-arg-bins=-Tlink-rp.x");
    println!("cargo:rustc-link-arg-bins=-Tdefmt.x");
}
