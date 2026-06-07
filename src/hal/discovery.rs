//! Hardware Discovery Logic for ELM.
//! Probes MMIO regions and indexes available Processing Units (PUs) into the registry.

use crate::hal::pu::{PuId, REGISTRY};
use crate::hal::rpi4::{Rpi4PmuPu, Rpi4NvmePu, Rpi4I2cPu, Rpi4UartPu};
use alloc::boxed::Box;

/// Probes the native hardware and registers discovered PUs.
/// This is called during the kernel boot sequence.
pub fn probe_hardware() {
    // In a full implementation, we would probe specific MMIO signatures
    // to verify hardware existence before registering.
    // For the RPi4 target, we register the known core PUs.

    // 1. PMU (Performance Monitoring Unit)
    REGISTRY.register(
        PuId(0),
        Box::new(Rpi4PmuPu)
    );

    // 2. NVMe Controller
    REGISTRY.register(
        PuId(1),
        Box::new(Rpi4NvmePu)
    );

    // 3. I2C Bus 0
    REGISTRY.register(
        PuId(2),
        Box::new(Rpi4I2cPu)
    );

    // 4. Debug UART
    REGISTRY.register(
        PuId(3),
        Box::new(Rpi4UartPu)
    );
}
