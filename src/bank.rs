extern crate alloc;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use crate::memory::pocket::{Pocket, ExperiencePayload};

pub struct MemoryBank {
    /// BTreeMap is used instead of HashMap because it is available in no_std alloc
    pub pockets: BTreeMap<u64, Pocket>,
    /// Maps a Cluster ID to a list of Pocket IDs
    pub clusters: BTreeMap<u64, Vec<u64>>,
    
    next_cluster_id: u64,
    
    // MVP Hyperparameters
    pub merge_threshold: f32,
    pub compress_size: usize,
    pub max_delta_for_rule: f32,
}

impl MemoryBank {
    pub fn new() -> Self {
        Self {
            pockets: BTreeMap::new(),
            clusters: BTreeMap::new(),
            next_cluster_id: 1,
            merge_threshold: 0.85,
            compress_size: 5,
            max_delta_for_rule: 0.2,
        }
    }

    /// Context-Sensitive Similarity Engine (Bare-Metal implementation)
    pub fn calculate_similarity(&self, p1: &Pocket, p2: &Pocket) -> f32 {
        // Phase 1 Bootstrapping: Actions must match perfectly to even be considered similar.
        if p1.payload.action != p2.payload.action {
            return 0.0;
        }

        // Calculate squared distance for Focal State (avoids needing a sqrt function from libm)
        let dx = (p1.payload.state_focal.position_x - p2.payload.state_focal.position_x) as f32;
        let dy = (p1.payload.state_focal.position_y - p2.payload.state_focal.position_y) as f32;
        let focal_dist_sq = (dx * dx) + (dy * dy);
        
        // Convert distance to a similarity score [0.0, 1.0]
        let sim_focal = 1.0 / (1.0 + focal_dist_sq);

        // Calculate similarity for Self State (Hardware Proprioception)
        let temp_diff = p1.payload.state_self.core_temp - p2.payload.state_self.core_temp;
        let temp_dist_sq = temp_diff * temp_diff;
        let sim_self = 1.0 / (1.0 + temp_dist_sq);

        // Dynamic Weights: 70% external attention, 30% internal hardware state
        let w_focal = 0.70;
        let w_self = 0.30;

        (w_focal * sim_focal) + (w_self * sim_self)
    }

    /// Ingests a new experience, quantizes it, links it, and checks for compression.
    pub fn evaluate_and_store(&mut self, mut new_pocket: Pocket) {
        // 1. Delta-Driven Quantization
        if new_pocket.payload.delta > 0.8 {
            new_pocket.quantization_level = 0; // High precision for pain/surprise
            new_pocket.significance = 1.0;
        } else {
            new_pocket.quantization_level = 1; // Reduced precision for expected outcomes
            new_pocket.significance = 0.5;
        }

        // 2. Nearest Neighbor Search (Linear scan of Tier 0 pockets for MVP)
        let mut best_sim = 0.0;
        let mut best_match_id: Option<u64> = None;

        for (id, existing_pocket) in self.pockets.iter() {
            if existing_pocket.compression_tier == 0 {
                let sim = self.calculate_similarity(&new_pocket, existing_pocket);
                if sim > best_sim {
                    best_sim = sim;
                    best_match_id = Some(*id);
                }
            }
        }

        // 3. Clustering Logic
        if best_sim >= self.merge_threshold {
            if let Some(target_id) = best_match_id {
                let mut target_cluster_id = self.pockets.get(&target_id).unwrap().cluster_id;

                // If the target doesn't have a cluster yet, create one
                if target_cluster_id.is_none() {
                    target_cluster_id = Some(self.next_cluster_id);
                    self.clusters.insert(self.next_cluster_id, alloc::vec![target_id]);
                    
                    // Update the target pocket in the bank
                    if let Some(p) = self.pockets.get_mut(&target_id) {
                        p.cluster_id = target_cluster_id;
                    }
                    self.next_cluster_id += 1;
                }

                // Assign the new pocket to the cluster
                new_pocket.cluster_id = target_cluster_id;
                if let Some(cluster_list) = self.clusters.get_mut(&target_cluster_id.unwrap()) {
                    cluster_list.push(new_pocket.id);
                }

                // Check if this cluster is ready to become a Rule
                self.check_compression_trigger(target_cluster_id.unwrap());
            }
        }

        // 4. Store the pocket in bare-metal RAM
        self.pockets.insert(new_pocket.id, new_pocket);
    }

    /// Evaluates a cluster to see if it has reached the threshold to become a Tier 1 Semantic Rule
    fn check_compression_trigger(&mut self, cluster_id: u64) {
        if let Some(pocket_ids) = self.clusters.get(&cluster_id) {
            if pocket_ids.len() < self.compress_size {
                return; // Not enough experiences yet
            }

            let mut total_delta = 0.0;
            for id in pocket_ids.iter() {
                if let Some(pocket) = self.pockets.get(id) {
                    total_delta += pocket.payload.delta;
                }
            }
            
            let avg_delta = total_delta / (pocket_ids.len() as f32);

            // If the model is consistently predicting this correctly (low delta), compress it!
            if avg_delta <= self.max_delta_for_rule {
                self.compress_to_tier_1(cluster_id);
            }
        }
    }

    /// The crucible of learning: Converts Episodic Memory into a Semantic Rule.
    fn compress_to_tier_1(&mut self, cluster_id: u64) {
        // In a full implementation, this creates a fundamentally new abstracted object.
        // For the MVP, we flag the pockets as Tier 1, protecting them from decay
        // and making them officially part of the ELM's World Model.
        
        if let Some(pocket_ids) = self.clusters.get(&cluster_id) {
            for id in pocket_ids.iter() {
                if let Some(pocket) = self.pockets.get_mut(id) {
                    pocket.compression_tier = 1;
                }
            }
        }
    }
}
