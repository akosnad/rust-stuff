#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rust_stuff::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;
extern crate rlibc;

use core::panic::PanicInfo;
use rust_stuff::{hlt_loop, println};
use bootloader::BootInfo;

#[no_mangle] // don't mangle the name of this function
pub extern "C" fn _start(boot_info: &'static BootInfo) -> ! {
    use rust_stuff::allocator;
    use rust_stuff::memory::{self, BootInfoFrameAllocator};

    let mut mapper = unsafe { memory::init(boot_info.physical_memory_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };
    
    allocator::init_heap(&mut mapper, 1024 * 1024 * 4, &mut frame_allocator).expect("heap initialization failed");

    test_main();
    hlt_loop();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rust_stuff::test_panic_handler(info);
}

#[test_case]
fn test_println() {
    println!("test_println output");
}
