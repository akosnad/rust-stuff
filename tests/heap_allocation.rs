#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rust_stuff::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;
extern crate rlibc;

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use rust_stuff::hlt_loop;

entry_point!(main);

pub const HEAP_SIZE: usize = 1024 * 64;

fn main(boot_info: &'static BootInfo) -> ! {
    use rust_stuff::allocator;
    use rust_stuff::memory::{self, BootInfoFrameAllocator};

    let phys_mem_offset = x86_64::VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };
    allocator::init_heap(&mut mapper, HEAP_SIZE, &mut frame_allocator).expect("heap initialization failed");
    rust_stuff::init();

    test_main();
    hlt_loop();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    rust_stuff::test_panic_handler(info)
}

use alloc::{boxed::Box, vec::Vec};

#[test_case]
fn simple_allocation() {
    let heap_value = Box::new(41);
    assert_eq!(*heap_value, 41);
}

#[test_case]
fn large_vec() {
    let n = 1000;
    let mut vec = Vec::new();
    for i in 0..n {
        vec.push(i);
    }
    assert_eq!(vec.iter().sum::<u64>(), (n - 1) * n / 2);
}

#[test_case]
fn many_boxes() {
    for i in 0..10_000 {
        let x = Box::new(i);
        assert_eq!(*x, i);
    }
}

#[test_case]
fn many_boxes_long_lived() {
    let long_lived = Box::new(1);
    for i in 0..HEAP_SIZE {
        let x = Box::new(i);
        assert_eq!(*x, i);
    }
    assert_eq!(*long_lived, 1);
}