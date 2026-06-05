#![no_std]
#![no_main]
// DELETE: #![feature(asm_const)]

extern crate alloc;

use bootloader::{entry_point, BootInfo};
use linked_list_allocator::LockedHeap;
use core::panic::PanicInfo;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

// Import our modules
mod vga_buffer;
use elm_kernel::agent::ELMAgent;

/// The bare-metal panic handler. 
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

// Hand control to the bootloader
entry_point!(kernel_main);

fn kernel_main(_boot_info: &'static BootInfo) -> ! {
    // 1. Initialize the Screen
    println!("Booting ELM Kernel (v0.1)...");
    
    // 2. Initialize the Heap in raw RAM
    const HEAP_START: usize = 0x_0010_0000;
    const HEAP_SIZE: usize = 100 * 1024; 
    
    unsafe {
        ALLOCATOR.lock().init(HEAP_START as *mut u8, HEAP_SIZE);
    }
    println!("Memory Allocation: OK");

    // 3. Boot the ELM
    let mut elm = ELMAgent::new();
    println!("ELM Agent Initialized. Entering Exploration Loop.");
    println!("------------------------------------------------");

    // 4. The Continuous Hardware Loop
    let action_sequence: [u8; 4] = [0, 1, 2, 3]; 
    let mut cycle_count = 0;

    loop {
        let action = action_sequence[cycle_count % 4];
        let action_str = match action {
            0 => "NORTH", 1 => "EAST", 2 => "SOUTH", 3 => "WEST", _ => "UNKNOWN"
        };

        // Execute action and get delta
        let delta = elm.step(action);

        // Print the cognitive state
        if delta > 0.8 {
            println!("Cycle {}: Action: {:<5} | Delta: {:.2} [SURPRISE/NOVELTY]", cycle_count, action_str, delta);
        } else if delta < 0.2 {
            println!("Cycle {}: Action: {:<5} | Delta: {:.2} [EXPECTED]", cycle_count, action_str, delta);
        }

        cycle_count += 1;
        
        // Slow down the hardware loop so we can actually read the screen
        for _ in 0..10_000_000 {
            unsafe { core::arch::asm!("nop"); } 
        }
    }
}
