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

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    use rust_stuff::allocator;
    use rust_stuff::memory::{self, BootInfoFrameAllocator};
    use rust_stuff::task::{Task, executor::Executor, keyboard, mouse, term, canvasgame};
    use rust_stuff::peripheral::{ISubject, keyboard::Keyboard, mouse::Mouse};
    use rust_stuff::vga::term::TERM_INPUT;

    crate::init();
    
    let phys_mem_offset = x86_64::VirtAddr::new(boot_info.physical_memory_offset.into_option().expect("no physical_memory_offset"));
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_regions) };
    
    allocator::init_heap(&mut mapper, 1024 * 1024 * 16, &mut frame_allocator).expect("heap initialization failed");
    
    #[cfg(test)]
    test_main();

    log::info!("Welcome");

    let mut keyboard = Keyboard::new();
    keyboard.attach(&*TERM_INPUT);

    let mut mouse = Mouse::new();
    mouse.attach(&*TERM_INPUT);

    let mut executor = Executor::new();
    executor.spawn(Task::new(term::process_buffer()));
    executor.spawn(Task::new(keyboard::process_keypresses(keyboard)));
    executor.spawn(Task::new(mouse::process_states(mouse)));
    executor.spawn(Task::new(canvasgame::run()));
    executor.run();
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    use rust_stuff::QemuExitCode;

    rust_stuff::serial_println!("{}", info);
    rust_stuff::println!("{}", info);
    x86_64::instructions::interrupts::disable();
    rust_stuff::exit_qemu(QemuExitCode::Failed);
    rust_stuff::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rust_stuff::test_panic_handler(info)
}
