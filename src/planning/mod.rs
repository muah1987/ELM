//! Planning Layer
//! Responsible for selecting the best action to achieve a target goal.

pub mod engine;

pub struct Goal {
    pub target_focal: [f32; 4],
    pub threshold: f32,
}

impl Goal {
    pub fn is_satisfied(&self, current: &[f32; 4]) -> bool {
        let mut dist_sq = 0.0;
        for i in 0..4 {
            let diff = self.target_focal[i] - current[i];
            dist_sq += diff * diff;
        }
        dist_sq < self.threshold
    }
}
