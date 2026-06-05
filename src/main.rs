#![no_std]
#![no_main]
#![feature(asm_const)]

extern crate alloc;

// In a bare-metal environment, we must provide our own memory allocator 
// because we don't have an OS to manage RAM for our BTreeMaps and Vecs.
use linked_list_allocator::LockedHeap;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

use core::panic::PanicInfo;

mod memory;
mod bank;
mod environment;
mod engine; // Previously loop.rs

use engine::ELMAgent;

/// The bare-metal panic handler. If the ELM crashes, the hardware halts.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // In a mature kernel, we would write the panic info to the VGA text buffer here.
    loop {}
}

/// The entry point. The bootloader jumps to this exact memory address.
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // 1. Initialize the Heap in raw RAM
    // We arbitrarily allocate 1MB of memory starting at a safe physical address.
    const HEAP_START: usize = 0x_0010_0000;
    const HEAP_SIZE: usize = 100 * 1024; // 100 KiB for the MVP Memory Bank
    
    unsafe {
        ALLOCATOR.lock().init(HEAP_START as *mut u8, HEAP_SIZE);
    }

    // 2. Boot the ELM
    let mut elm = ELMAgent::new();

    // 3. The Continuous Hardware Loop
    // For the MVP, we feed it a hardcoded exploration pattern: North, East, South, West.
    let action_sequence: [u8; 4] = [0, 1, 2, 3]; 
    let mut cycle_count = 0;

    loop {
        let action = action_sequence[cycle_count % 4];

        // The ELM predicts, acts, and returns its mathematical surprise
        let delta = elm.step(action);

        // If Delta is high, the ELM experienced "pain" or "novelty".
        // It has automatically stored this in Tier 0 memory at full precision.
        if delta > 0.8 {
            // In a mature kernel, we trigger an interrupt or print to the screen here.
        }

        cycle_count += 1;
        
        // Prevent melting the CPU. Yield cycles until the next "tick".
        for _ in 0..100_000 {
            unsafe { core::arch::asm!("nop"); } // "No Operation" hardware instruction
        }
    }
}
