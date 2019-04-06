#![allow(clippy::empty_loop)]

use core::panic::PanicInfo;
use core::ptr;

use crate::exception_vectors;
use crate::mmu;
use crate::tegra210;

use core::fmt::Write;

#[macro_export]
macro_rules! entry {
    ($path:path) => {
        #[export_name = "main"]
        pub unsafe fn __main() -> () {
            // type check the given path
            let f: fn() -> () = $path;

            f()
        }
    };
}

#[panic_handler]
fn panic(panic_info: &PanicInfo<'_>) -> ! {
    let mut uart_a = &mut tegra210::uart::UART::A;
    writeln!(&mut uart_a, "PANIC: {}\r", panic_info).ok();
    unsafe {
        reboot_to_rcm();
    };
    loop {}
}

extern "C" {
    static mut __start_bss__: u8;
    static mut __end_bss__: u8;
    static _stack_bottom: u8;
    static _stack_top: u8;
}

#[no_mangle]
pub unsafe extern "C" fn reboot_to_rcm() {
    asm!(
        "mov x1, xzr
    mov w2, #0x2
    movz x1, 0xE450
    movk x1, #0x7000, lsl 16
    str w2, [x1]
    movz x1, #0xE400
    movk x1, #0x7000, lsl 16
    ldr w0, [x1]
    orr w0, w0, #0x10
    str w0, [x1]"
    );
}

#[link_section = ".text.boot"]
//#[naked]
#[no_mangle]
pub unsafe extern "C" fn _start() -> ! {
    asm!("mov sp, $0
     b _start_with_stack"
    :: "r"(&_stack_top as *const u8 as usize) :: "volatile");
    core::intrinsics::unreachable()
}

#[no_mangle]
pub unsafe extern "C" fn _start_with_stack() -> ! {
    // Clean .bss
    // FIXME: Will not work when we will want relocation
    let count = &__end_bss__ as *const u8 as usize - &__start_bss__ as *const u8 as usize;
    ptr::write_bytes(&mut __start_bss__ as *mut u8, 0, count);

    exception_vectors::setup();
    mmu::setup();

    // Call user entry point
    extern "Rust" {
        fn main() -> ();
    }

    main();
    reboot_to_rcm();

    loop {}
}
