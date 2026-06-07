/// StateSelf represents physical hardware proprioception.
/// Read directly from the native PU Registry.
use alloc::vec::Vec;
use crate::memory::pocket::PainSensor;
use crate::hal::pu::PuId;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StateSelf {
    pub cpu_cycles: u64,          // Absolute time in raw hardware ticks
    pub core_temp: f32,           // Read from thermal sensor
    pub page_fault_count: u32,    // MMU distress metric
    pub inference_latency: u64,   // Cycle cost of the last prediction
}

impl StateSelf {
    /// Reads the current hardware state and returns both the state and the pain status.
    pub fn read_native_with_pain() -> (Self, PainSensor) {
        use crate::hal::pu::{PuId, REGISTRY};

        // Read thermal data from BCM2711 register 0xFE212058
        const THERMAL_SENSOR_ADDR: usize = 0xFE212058;
        const THERMAL_OFFSET: f32 = -709.0;
        const THERMAL_DIVISOR: f32 = 4.26;

        let raw = unsafe {
            core::ptr::read_volatile(THERMAL_SENSOR_ADDR as *const u32)
        };
        let raw_temp = (raw & 0x3FF) as f32;
        let temp = (raw_temp + THERMAL_OFFSET) / THERMAL_DIVISOR;

        // Pain Calculation
        let mut pain_active = false;
        let mut magnitude = 0.0f32;

        if temp > 80.0 {
            pain_active = true;
            magnitude += 0.5;
        }

        let pain = PainSensor {
            pain_magnitude: magnitude.min(1.0),
            pain_active,
        };

        let state = Self {
            cpu_cycles: REGISTRY.read(PuId(0), 0) as u64,
            core_temp: temp,
            page_fault_count: 0,
            inference_latency: 0,
        };

        (state, pain)
    }

    /// Returns a list of all indexed PUs for health monitoring.
    pub fn get_hardware_health() -> Vec<PuId> {
        use crate::hal::pu::REGISTRY;
        REGISTRY.list_pushed()
    }

    /// Fallback for simulation/testing
    pub fn read_current_state(current_cycles: u64, temp: f32, faults: u32) -> Self {
        Self {
            cpu_cycles: current_cycles,
            core_temp: temp,
            page_fault_count: faults,
            inference_latency: 0,
        }
    }
}
