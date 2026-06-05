#![no_std]

extern crate alloc;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use crate::memory::pocket::Pocket;
use crate::memory::similarity::SimilarityEngine;

pub struct MemoryBank {
    pub pockets: BTreeMap<u64, Pocket>,
    pub clusters: BTreeMap<u64, Vec<u64>>,
    pub similarity: SimilarityEngine,
    
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
            next_cluster_id: 1,
            merge_threshold: 0.85,
            compress_size: 5,
            max_delta_for_rule: 0.2,
        }
    }

    pub fn evaluate_and_store(&mut self, mut new_pocket: Pocket) {
        if new_pocket.payload.delta > 0.8 {
            new_pocket.quantization_level = 0; 
            new_pocket.significance = 1.0;
        } else {
            new_pocket.quantization_level = 1; 
            new_pocket.significance = 0.5;
        }

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
            for id in pocket_ids.iter() {
                if let Some(pocket) = self.pockets.get_mut(id) {
                    pocket.compression_tier = 1;
                }
            }
        }
    }
}
