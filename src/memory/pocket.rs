extern crate alloc;
use alloc::vec::Vec;

const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;

pub fn fnv1a_hash(bytes: &[u8]) -> u64 {
    let mut hash = FNV_OFFSET_BASIS;
    for byte in bytes {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct StateAmbient {
    pub grid_temp: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct StateFocal {
    pub position_x: i32,
    pub position_y: i32,
    pub touching_wall: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ValenceSource {
    Pain,
    Punishment,
    Relief,
    Reward,
    Neutral,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Valence {
    pub value: f32,          // Signed scalar [-1.0, 1.0]
    pub source: ValenceSource,
    pub decay_modifier: f32, // How fast this memory loses significance
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PainSensor {
    pub pain_magnitude: f32,
    pub pain_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperiencePayload {
    pub normalized_self: [f32; 4],    // [temp, cycles, faults, latency]
    pub normalized_ambient: [f32; 1], // [temp]
    pub normalized_focal: [f32; 4],   // [x, y, touch, reserve]
    pub action: u8,
    pub normalized_outcome: Option<[f32; 4]>,
    pub delta: f32,
    pub valence: Valence,
    pub pain_at_time: PainSensor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pocket {
    pub id: u64,
    pub cluster_id: Option<u64>,
    pub payload: ExperiencePayload,
    pub timestamp_cycles: u64,
    pub quantization_level: u8,
    pub significance: f32,
    pub compression_tier: u8,
    pub edges: Vec<u64>,
}

impl Pocket {
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

    fn generate_id(&self) -> u64 {
        let mut data_to_hash: Vec<u8> = Vec::with_capacity(20);
        data_to_hash.extend_from_slice(&self.timestamp_cycles.to_le_bytes());
        // Use the normalized focal state for the hash
        data_to_hash.extend_from_slice(&self.payload.normalized_focal[0].to_be_bytes());
        data_to_hash.extend_from_slice(&self.payload.normalized_focal[1].to_be_bytes());
        data_to_hash.push(self.payload.action);
        fnv1a_hash(&data_to_hash)
    }
}
