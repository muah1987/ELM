use crate::memory::pocket::Pocket;

/// The Context-Sensitive Similarity Engine
pub struct SimilarityEngine {
    pub w_focal: f32,
    pub w_self: f32,
}

impl SimilarityEngine {
    pub fn new() -> Self {
        Self {
            w_focal: 0.70,
            w_self: 0.30,
        }
    }

    /// Context-Sensitive context weighting could be injected here later
    pub fn calculate(&self, p1: &Pocket, p2: &Pocket) -> f32 {
        // Phase 1 Bootstrapping: Actions must match
        if p1.payload.action != p2.payload.action {
            return 0.0;
        }

        // Squared Euclidean distance for Focal State (no float-math sqrt needed)
        let dx = p1.payload.normalized_focal[0] - p2.payload.normalized_focal[0];
        let dy = p1.payload.normalized_focal[1] - p2.payload.normalized_focal[1];
        let focal_dist_sq = (dx * dx) + (dy * dy);
        let sim_focal = 1.0 / (1.0 + focal_dist_sq);

        // Hardware Proprioception difference
        let temp_diff = p1.payload.normalized_self[0] - p2.payload.normalized_self[0];
        let temp_dist_sq = temp_diff * temp_diff;
        let sim_self = 1.0 / (1.0 + temp_dist_sq);

        (self.w_focal * sim_focal) + (self.w_self * sim_self)
    }
}
