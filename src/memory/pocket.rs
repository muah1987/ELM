extern crate alloc;
use alloc::vec::Vec;
use crate::sensors::state_self::StateSelf;

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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StateAmbient {
    pub grid_temp: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StateFocal {
    pub position_x: i32,
    pub position_y: i32,
    pub touching_wall: bool,
}

#[derive(Debug, Clone)]
pub struct ExperiencePayload {
    pub state_self: StateSelf,
    pub state_ambient: StateAmbient,
    pub state_focal: StateFocal,
    pub action: u8,               
    pub outcome: Option<StateFocal>, 
    pub delta: f32,               
}

#[derive(Debug, Clone)]
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
        data_to_hash.extend_from_slice(&self.payload.state_focal.position_x.to_le_bytes());
        data_to_hash.extend_from_slice(&self.payload.state_focal.position_y.to_le_bytes());
        data_to_hash.push(self.payload.action);
        fnv1a_hash(&data_to_hash)
    }
}
