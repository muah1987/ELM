use crate::memory::bank::MemoryBank;
use crate::memory::pocket::{ExperiencePayload, StateFocal};
use alloc::vec::Vec; // Ensure you are importing Vec explicitly here

/// The internal World Model. 
/// Queries the Memory Bank to project the future based on the past.
pub struct WorldModel;

impl WorldModel {
    /// Retrieves Tier 1 Semantic Rules to predict the outcome of an intended action.
    pub fn predict_outcome(memory: &MemoryBank, payload: &ExperiencePayload) -> Option<StateFocal> {
        let mut best_sim = 0.0;
        let mut predicted_state: Option<StateFocal> = None;

        // Iterate through memory looking strictly for Semantic Rules (Tier 1)
        for (_, pocket) in memory.pockets.iter() {
            if pocket.compression_tier == 1 {
                // We create a temporary dummy pocket to utilize our similarity engine
                // comparing the intended payload against the stored rule payload.
                let dummy_current = crate::memory::pocket::Pocket {
                    id: 0,
                    cluster_id: None,
                    payload: payload.clone(),
                    timestamp_cycles: 0,
                    quantization_level: 0,
                    significance: 0.0,
                    compression_tier: 0,
                    edges: alloc::vec::Vec::new(),
                };

                let sim = memory.similarity.calculate(&dummy_current, pocket);
                
                // If we find a highly similar past rule for this exact action
                if sim > 0.90 && pocket.payload.action == payload.action {
                    best_sim = sim;
                    predicted_state = pocket.payload.outcome;
                }
            }
        }
        
        predicted_state
    }
}
