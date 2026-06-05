/// StateSelf represents physical hardware proprioception.
/// Read directly from CPU Time Stamp Counters (TSC), MSRs, and MMU.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StateSelf {
    pub cpu_cycles: u64,          // Absolute time in raw hardware ticks
    pub core_temp: f32,           // Read from thermal MSR
    pub page_fault_count: u32,    // MMU distress metric
    pub inference_latency: u64,   // Cycle cost of the last prediction
}

impl StateSelf {
    /// Initializes a baseline hardware state. 
    /// In a mature kernel, this reads raw registers: `core::arch::x86_64::_rdtsc()`
    pub fn read_current_state(current_cycles: u64, temp: f32, faults: u32) -> Self {
        Self {
            cpu_cycles: current_cycles,
            core_temp: temp,
            page_fault_count: faults,
            inference_latency: 0, 
        }
    }
}
