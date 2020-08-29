#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rust_stuff::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;
extern crate rlibc;

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use rust_stuff::init;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use rust_stuff::allocator;
    use rust_stuff::memory::{self, BootInfoFrameAllocator};
    use rust_stuff::task::{Task, executor::Executor, keyboard, term};

    crate::init();
    
    let mut mapper = unsafe { memory::init(boot_info.physical_memory_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };
    
    allocator::init_heap(&mut mapper, 1024 * 1024 * 4, &mut frame_allocator).expect("heap initialization failed");
    
    #[cfg(test)]
    test_main();

    let mut executor = Executor::new();
    executor.spawn(Task::new(term::process_buffer()));
    executor.spawn(Task::new(keyboard::process_keypresses()));
    executor.run();
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rust_stuff::serial_println!("{}", info);
    rust_stuff::println!("{}", info);
    x86_64::instructions::interrupts::disable();
    rust_stuff::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rust_stuff::test_panic_handler(info)
}
