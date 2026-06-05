#![no_std]

extern crate alloc;
use alloc::vec::Vec;

/// FNV-1a bare-metal hashing parameters
const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;

/// A lightweight, bare-metal hash to generate Pocket IDs without OS crypto libs
fn fnv1a_hash(bytes: &[u8]) -> u64 {
    let mut hash = FNV_OFFSET_BASIS;
    for byte in bytes {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

/// StateSelf represents the physical hardware proprioception.
/// Read directly from CPU Time Stamp Counters (TSC), MSRs, and MMU.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StateSelf {
    pub cpu_cycles: u64,          // Absolute time in raw hardware ticks
    pub core_temp: f32,           // Read from thermal MSR
    pub page_fault_count: u32,    // MMU distress metric
    pub inference_latency: u64,   // Cycle cost of the last prediction
}

/// StateAmbient represents the background grid environment.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StateAmbient {
    pub grid_temp: f32,
}

/// StateFocal represents the agent's immediate attention and physical boundaries.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StateFocal {
    pub position_x: i32,
    pub position_y: i32,
    pub touching_wall: bool,
}

/// The core experience tuple. 
#[derive(Debug, Clone)]
pub struct ExperiencePayload {
    pub state_self: StateSelf,
    pub state_ambient: StateAmbient,
    pub state_focal: StateFocal,
    pub action: u8,               // 0: N, 1: E, 2: S, 3: W
    pub outcome: Option<StateFocal>, // None if the action hasn't executed yet
    pub delta: f32,               // 1.0 = Maximum surprise / Pain
}

/// The fundamental unit of memory in the ELM.
#[derive(Debug, Clone)]
pub struct Pocket {
    pub id: u64,                  // FNV-1a hash instead of string for memory efficiency
    pub cluster_id: Option<u64>,  // Groups similar pockets for Tier 1 compression
    pub payload: ExperiencePayload,
    pub timestamp_cycles: u64,    // CPU TSC at the exact moment of creation
    pub quantization_level: u8,   // 0 = Float32 (Novel), 1 = Float16 (Familiar)
    pub significance: f32,        // Drives retention and prevents overwriting
    pub compression_tier: u8,     // 0 = Episodic, 1 = Semantic Rule
    pub edges: Vec<u64>,          // Pointers to related Pocket IDs
}

impl Pocket {
    /// Constructs a new Pocket and automatically generates its deterministic bare-metal hash.
    pub fn new(payload: ExperiencePayload, current_cycles: u64) -> Self {
        let mut pocket = Pocket {
            id: 0, 
            cluster_id: None,
            payload,
            timestamp_cycles: current_cycles,
            quantization_level: 0,
            significance: 1.0,
            compression_tier: 0,
            edges: Vec::new(),
        };
        
        pocket.id = pocket.generate_id();
        pocket
    }

    /// Derives a unique ID based on the exact moment in time and physical location.
    fn generate_id(&self) -> u64 {
        // Serialize the critical uniqueness factors into a byte array
        let mut data_to_hash: Vec<u8> = Vec::with_capacity(20);
        
        // Push timestamp
        data_to_hash.extend_from_slice(&self.timestamp_cycles.to_le_bytes());
        // Push X coord
        data_to_hash.extend_from_slice(&self.payload.state_focal.position_x.to_le_bytes());
        // Push Y coord
        data_to_hash.extend_from_slice(&self.payload.state_focal.position_y.to_le_bytes());
        // Push action
        data_to_hash.push(self.payload.action);

        fnv1a_hash(&data_to_hash)
    }
}
