#![no_std]
#![no_main]

extern crate alloc;

use bootloader::{entry_point, BootInfo};
use linked_list_allocator::LockedHeap;
use core::panic::PanicInfo;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

// Import the ELM Library you just built!
use elm_kernel::world::grid::GridWorld;
use elm_kernel::memory::bank::MemoryBank;
use elm_kernel::agent::ELMAgent;
use elm_kernel::println;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

entry_point!(kernel_main);

fn kernel_main(_boot_info: &'static BootInfo) -> ! {
    println!("Booting ELM Thermal Maze MVP...");
    
    // Initialize Heap
    const HEAP_START: usize = 0x_0010_0000;
    const HEAP_SIZE: usize = 100 * 1024; 
    unsafe { ALLOCATOR.lock().init(HEAP_START as *mut u8, HEAP_SIZE); }

    // Initialize ELM Components from the library
    let mut env = GridWorld::new();
    let mut memory = MemoryBank::new();
    
    println!("ELM Agent Initialized. Entering Exploration Loop.");
    
    // The bare-metal MVP loop logic goes here...
    
    loop {
        // Prevent melting the CPU
        for _ in 0..10_000_000 { unsafe { core::arch::asm!("nop"); } }
    }
}
