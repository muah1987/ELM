extern crate alloc;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use crate::memory::pocket::{ExperiencePayload, Pocket, Valence, ValenceSource};
use crate::memory::similarity::SimilarityEngine;
use crate::hal::pu::{PuId, REGISTRY};
use postcard::{to_slice, from_bytes};
use serde::{Serialize, Deserialize};

/// A mapping of State-Action pairs to their expected hedonic outcome.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HedonicGradient {
    pub expected_valence: BTreeMap<u64, f32>, // Key: hash of (state, action), Value: Moving avg of valence
}

impl HedonicGradient {
    pub fn update(&mut self, key: u64, observed_valence: f32) {
        let entry = self.expected_valence.entry(key).or_insert(0.0);
        *entry = (*entry * 0.8) + (observed_valence * 0.2);
    }

    pub fn get_expected(&self, key: u64) -> f32 {
        *self.expected_valence.get(&key).unwrap_or(&0.0)
    }
}

pub struct MemoryBank {
    pub pockets: BTreeMap<u64, Pocket>,
    pub clusters: BTreeMap<u64, Vec<u64>>,
    pub similarity: SimilarityEngine,
    pub hedonic: HedonicGradient,

    next_cluster_id: u64,
    pub merge_threshold: f32,
    pub compress_size: usize,
    pub max_delta_for_rule: f32,
}

impl MemoryBank {
    pub fn new() -> Self {
        Self {
            pockets: BTreeMap::new(),
            clusters: BTreeMap::new(),
            similarity: SimilarityEngine::new(),
            hedonic: HedonicGradient::default(),
            next_cluster_id: 1,
            merge_threshold: 0.85,
            compress_size: 5,
            max_delta_for_rule: 0.2,
        }
    }

    /// Save the entire memory bank to NVMe storage
    pub fn save(&self) -> Result<(), crate::hal::HalError> {
        let mut current_lba = 0u64;
        let mut buffer = [0u8; 4096];
        let storage_id = PuId(1);

        // 1. Save Metadata
        let meta = [self.merge_threshold, self.max_delta_for_rule];
        if let Ok(bytes) = to_slice(&meta, &mut buffer) {
            REGISTRY.write_block(storage_id, current_lba, bytes)?;
        }
        current_lba += 1;

        // 2. Save Pockets
        for (_, pocket) in self.pockets.iter() {
            if let Ok(bytes) = to_slice(pocket, &mut buffer) {
                REGISTRY.write_block(storage_id, current_lba, bytes)?;
                current_lba += 1;
            }
        }

        REGISTRY.flush(storage_id)?;
        Ok(())
    }

    /// Load memory bank from NVMe storage
    pub fn load(&mut self) -> Result<(), crate::hal::HalError> {
        let mut current_lba = 0u64;
        let mut buffer = [0u8; 4096];
        let storage_id = PuId(1);

        // 1. Load Metadata
        REGISTRY.read_block(storage_id, current_lba, &mut buffer)?;
        if let Ok(meta) = from_bytes::<[f32; 2]>(&buffer) {
            self.merge_threshold = meta[0];
            self.max_delta_for_rule = meta[1];
        }
        current_lba += 1;

        // 2. Load Pockets (simplified: read until failure or limit)
        for _ in 0..1000 {
            if REGISTRY.read_block(storage_id, current_lba, &mut buffer).is_err() { break; }
            if let Ok(pocket) = from_bytes::<Pocket>(&buffer) {
                self.pockets.insert(pocket.id, pocket);
            }
            current_lba += 1;
        }

        Ok(())
    }

    /// Context-Sensitive Similarity Engine (Bare-Metal implementation)
    pub fn calculate_similarity(&self, p1: &Pocket, p2: &Pocket) -> f32 {
        self.similarity.calculate(p1, p2)
    }

    pub fn evaluate_and_store(&mut self, mut new_pocket: Pocket) {
        let valence_abs = new_pocket.payload.valence.value.abs();
        let delta_sq = new_pocket.payload.delta * new_pocket.payload.delta;

        // Significance = clamp((Delta² * 0.6) + (|Valence| * 0.4), 0, 1)
        let mut significance = (delta_sq * 0.6) + (valence_abs * 0.4);

        // Amplification for high-impact emotional states
        if new_pocket.payload.valence.source == crate::memory::pocket::ValenceSource::Pain
           || new_pocket.payload.valence.source == crate::memory::pocket::ValenceSource::Relief {
            significance *= 1.5;
        }

        new_pocket.significance = significance.clamp(0.0, 1.0);

        let mut best_sim = 0.0;
        let mut best_match_id: Option<u64> = None;

        for (id, existing_pocket) in self.pockets.iter() {
            if existing_pocket.compression_tier == 0 {
                let sim = self.similarity.calculate(&new_pocket, existing_pocket);
                if sim > best_sim {
                    best_sim = sim;
                    best_match_id = Some(*id);
                }
            }
        }

        if best_sim >= self.merge_threshold {
            if let Some(target_id) = best_match_id {
                let mut target_cluster_id = self.pockets.get(&target_id).unwrap().cluster_id;

                if target_cluster_id.is_none() {
                    target_cluster_id = Some(self.next_cluster_id);
                    self.clusters.insert(self.next_cluster_id, alloc::vec![target_id]);
                    if let Some(p) = self.pockets.get_mut(&target_id) {
                        p.cluster_id = target_cluster_id;
                    }
                    self.next_cluster_id += 1;
                }

                new_pocket.cluster_id = target_cluster_id;
                if let Some(cluster_list) = self.clusters.get_mut(&target_cluster_id.unwrap()) {
                    cluster_list.push(new_pocket.id);
                }

                self.check_compression_trigger(target_cluster_id.unwrap());
            }
        }

        self.pockets.insert(new_pocket.id, new_pocket);
    }

    fn check_compression_trigger(&mut self, cluster_id: u64) {
        if let Some(pocket_ids) = self.clusters.get(&cluster_id) {
            if pocket_ids.len() < self.compress_size { return; }

            let mut total_delta = 0.0;
            for id in pocket_ids.iter() {
                if let Some(pocket) = self.pockets.get(id) {
                    total_delta += pocket.payload.delta;
                }
            }

            let avg_delta = total_delta / (pocket_ids.len() as f32);
            if avg_delta <= self.max_delta_for_rule {
                self.compress_to_tier_1(cluster_id);
            }
        }
    }

    fn compress_to_tier_1(&mut self, cluster_id: u64) {
        if let Some(pocket_ids) = self.clusters.get(&cluster_id) {
            // Calculate Centroid (Mean) of the cluster
            let mut sum_focal = [0.0f32; 4];
            let mut sum_ambient = [0.0f32; 1];
            let mut count = 0usize;

            for id in pocket_ids.iter() {
                if let Some(p) = self.pockets.get(id) {
                    for i in 0..4 { sum_focal[i] += p.payload.normalized_focal[i]; }
                    for i in 0..1 { sum_ambient[i] += p.payload.normalized_ambient[i]; }
                    count += 1;
                }
            }

            if count == 0 { return; }

            let centroid_focal = sum_focal.map(|v| v / count as f32);
            let centroid_ambient = sum_ambient.map(|v| v / count as f32);

            // Create the Tier 1 Rule (Centroid Pocket)
            let first_id = pocket_ids[0];
            let first_action = self.pockets.get(&first_id).map(|p| p.payload.action).unwrap_or(0);

            let rule_payload = crate::memory::pocket::ExperiencePayload {
                normalized_self: [0.0; 4],
                normalized_ambient: centroid_ambient,
                normalized_focal: centroid_focal,
                action: first_action,
                normalized_outcome: None,
                delta: 0.0, // Rules are blueprints, not experiences
                valence: crate::memory::pocket::Valence {
                    value: 0.0,
                    source: crate::memory::pocket::ValenceSource::Neutral,
                    decay_modifier: 1.0,
                },
                pain_at_time: crate::memory::pocket::PainSensor {
                    pain_magnitude: 0.0,
                    pain_active: false,
                },
            };

            let rule_pocket = Pocket::new(rule_payload, 0); // Timestamp 0 for rules
            let rule_id = rule_pocket.id;

            // Mark all contributing pockets as Tier 1
            for id in pocket_ids.iter() {
                if let Some(p) = self.pockets.get_mut(id) {
                    p.compression_tier = 1;
                }
            }

            // Store the rule and link the cluster to it
            self.pockets.insert(rule_id, rule_pocket);
        }
    }

    fn compress_to_tier_2(&mut self, rule_cluster_id: u64) {
        if let Some(rule_ids) = self.clusters.get(&rule_cluster_id) {
            let mut sum_focal = [0.0f32; 4];
            let mut count = 0usize;

            for id in rule_ids.iter() {
                if let Some(p) = self.pockets.get(id) {
                    if p.compression_tier == 1 {
                        for i in 0..4 { sum_focal[i] += p.payload.normalized_focal[i]; }
                        count += 1;
                    }
                }
            }

            if count == 0 { return; }
            let principle_focal = sum_focal.map(|v| v / count as f32);

            let principle_payload = crate::memory::pocket::ExperiencePayload {
                normalized_self: [0.0; 4],
                normalized_ambient: [0.0],
                normalized_focal: principle_focal,
                action: 0,
                normalized_outcome: None,
                delta: 0.0,
                valence: crate::memory::pocket::Valence {
                    value: 0.0,
                    source: crate::memory::pocket::ValenceSource::Neutral,
                    decay_modifier: 1.0,
                },
                pain_at_time: crate::memory::pocket::PainSensor {
                    pain_magnitude: 0.0,
                    pain_active: false,
                },
            };

            let principle_pocket = Pocket::new(principle_payload, 0);
            let p_id = principle_pocket.id;

            for id in rule_ids.iter() {
                if let Some(p) = self.pockets.get_mut(id) {
                    p.compression_tier = 2;
                }
            }

            self.pockets.insert(p_id, principle_pocket);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::pocket::{ExperiencePayload, Pocket};

    #[test]
    fn test_centroid_compression() {
        let mut bank = MemoryBank::new();
        bank.compress_size = 2;
        bank.merge_threshold = 0.1; // Force merge
        bank.max_delta_for_rule = 1.0; // Force compression

        let p1 = Pocket::new(ExperiencePayload {
            normalized_self: [0.0; 4],
            normalized_ambient: [0.0],
            normalized_focal: [0.1, 0.1, 0.0, 0.0],
            action: 0,
            normalized_outcome: None,
            delta: 0.1,
            valence: crate::memory::pocket::Valence {
                value: 0.0,
                source: crate::memory::pocket::ValenceSource::Neutral,
                decay_modifier: 1.0,
            },
            pain_at_time: crate::memory::pocket::PainSensor {
                pain_magnitude: 0.0,
                pain_active: false,
            },
        }, 100);

        let p2 = Pocket::new(ExperiencePayload {
            normalized_self: [0.0; 4],
            normalized_ambient: [0.0],
            normalized_focal: [0.3, 0.3, 0.0, 0.0],
            action: 0,
            normalized_outcome: None,
            delta: 0.1,
            valence: crate::memory::pocket::Valence {
                value: 0.0,
                source: crate::memory::pocket::ValenceSource::Neutral,
                decay_modifier: 1.0,
            },
            pain_at_time: crate::memory::pocket::PainSensor {
                pain_magnitude: 0.0,
                pain_active: false,
            },
        }, 200);

        bank.evaluate_and_store(p1);
        bank.evaluate_and_store(p2);

        // Verify a rule was created (tier 1)
        let has_rule = bank.pockets.values().any(|p| p.compression_tier == 1);
        assert!(has_rule, "Centroid rule should have been created");
    }
}
