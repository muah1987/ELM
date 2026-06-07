//! Planning Engine
//! Implements goal-directed behavior by simulating outcomes of possible actions.

use crate::memory::bank::MemoryBank;
use crate::prediction::engine::WorldModel;
use crate::planning::Goal;

pub struct PlanningEngine;

impl PlanningEngine {
    /// Selects the best action to move closer to the goal, biased by hedonic valence.
    pub fn plan_next_action(
        memory: &MemoryBank,
        current_payload: &crate::memory::pocket::ExperiencePayload,
        goal: &Goal
    ) -> u8 {
        let actions = [0, 1, 2, 3]; // North, South, East, West
        let mut best_action = 0;
        let mut max_score = f32::MIN;

        for &action in &actions {
            // Create a hypothetical payload for this action
            let hypothetical_payload = crate::memory::pocket::ExperiencePayload {
                action,
                normalized_outcome: None,
                ..*current_payload
            };

            // 1. Factual Confidence (Distance to goal)
            let mut confidence = 0.0;
            if let Some(predicted_outcome) = WorldModel::predict_outcome(memory, &hypothetical_payload) {
                let mut dist_sq = 0.0;
                for i in 0..4 {
                    let diff = predicted_outcome[i] - goal.target_focal[i];
                    dist_sq += diff * diff;
                }
                confidence = 1.0 / (1.0 + dist_sq);
            }

            // 2. Hedonic Bias (Expected Valence)
            // In a real implementation, we'd hash (state, action) to look up in MemoryBank::hedonic
            let expected_valence = 0.0; // Placeholder until key hashing is implemented

            // Total Score = (Confidence * 0.6) + (ExpectedValence * 0.4)
            let score = (confidence * 0.6) + (expected_valence * 0.4);

            if score > max_score {
                max_score = score;
                best_action = action;
            }
        }

        best_action
    }
}
