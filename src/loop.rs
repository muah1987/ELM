#![no_std]

use crate::memory::{ExperiencePayload, Pocket, StateSelf, StateFocal};
use crate::bank::MemoryBank;
use crate::environment::GridWorld;

pub struct ELMAgent {
    pub env: GridWorld,
    pub memory: MemoryBank,
    
    // Bare-metal hardware state mocks (In a real kernel, these read physical registers)
    hardware_cycles: u64,
    core_temp: f32,
    page_faults: u32,
}

impl ELMAgent {
    pub fn new() -> Self {
        Self {
            env: GridWorld::new(),
            memory: MemoryBank::new(),
            hardware_cycles: 0,
            core_temp: 45.0, // Resting temperature in Celsius
            page_faults: 0,
        }
    }

    /// Simulates reading from the CPU's hardware registers
    fn read_hardware_state(&mut self) -> StateSelf {
        // In a real OS-less environment, this would be: `core::arch::x86_64::_rdtsc()`
        self.hardware_cycles += 100; // Base cost of taking an action
        
        StateSelf {
            cpu_cycles: self.hardware_cycles,
            core_temp: self.core_temp,
            page_fault_count: self.page_faults,
            inference_latency: 0, // Calculated after prediction
        }
    }

    /// The World Model Query. 
    /// Searches for Tier 1 Rules that match the current state and intended action.
    fn predict_outcome(&mut self, payload: &ExperiencePayload) -> Option<StateFocal> {
        let start_cycles = self.hardware_cycles;
        
        let mut best_sim = 0.0;
        let mut predicted_state: Option<StateFocal> = None;

        // Iterate through memory looking for Semantic Rules (Tier 1)
        for (_, pocket) in self.memory.pockets.iter() {
            if pocket.compression_tier == 1 {
                let sim = self.memory.calculate_similarity(&pocket, &pocket); // Simplified for MVP
                
                // If we find a highly similar past rule for this exact action
                if sim > 0.90 && pocket.payload.action == payload.action {
                    best_sim = sim;
                    predicted_state = pocket.payload.outcome;
                }
            }
        }

        // Proprioception update: querying memory takes time (latency)
        self.hardware_cycles += 500; 
        
        predicted_state
    }

    /// The core cognitive loop. Executes one cycle of experience.
    pub fn step(&mut self, action: u8) -> f32 {
        let cycle_start = self.hardware_cycles;

        // 1. Capture Pre-Action States
        let (focal_state, ambient_state) = self.env.get_states();
        
        // Simulate hardware exertion: complex actions or moving heat increases temp
        self.core_temp += 0.1; 
        let mut self_state = self.read_hardware_state();

        // 2. Assemble Pre-Action Payload
        let mut payload = ExperiencePayload {
            state_self: self_state,
            state_ambient: ambient_state,
            state_focal: focal_state,
            action,
            outcome: None, 
            delta: 1.0, // Defaults to max surprise
        };

        // 3. Predict the Future (Query World Model)
        let predicted_outcome = self.predict_outcome(&payload);

        // 4. Execute Physical Action in the Environment
        let actual_outcome = self.env.execute(action);
        payload.outcome = Some(actual_outcome);

        // 5. Calculate Proprioceptive Latency
        let cycle_end = self.hardware_cycles + 200; // Add execution cost
        self.hardware_cycles = cycle_end;
        payload.state_self.inference_latency = cycle_end - cycle_start;

        // 6. Calculate Delta (The mathematical measure of surprise)
        if let Some(predicted) = predicted_outcome {
            // Distance between predicted reality and actual reality
            let dx = (predicted.position_x - actual_outcome.position_x) as f32;
            let dy = (predicted.position_y - actual_outcome.position_y) as f32;
            let dist_sq = (dx * dx) + (dy * dy);
            
            // Normalize Delta to [0.0, 1.0] where 1.0 is totally unexpected
            // A distance of 0 means perfect prediction (Delta = 0.0)
            payload.delta = dist_sq / (1.0 + dist_sq); 
        } else {
            // Phase 0: If no prediction could be made, everything is surprising.
            payload.delta = 1.0; 
        }

        // 7. Hardware Pain Overrides
        // Even if the external world behaved as expected, if the body is failing, 
        // the experience must be flagged as highly significant.
        if payload.state_self.core_temp > 85.0 {
            // Thermal throttling threshold reached! 
            payload.delta = 1.0; 
        }

        // 8. Package and Store
        let current_timestamp = self.hardware_cycles;
        let new_pocket = Pocket::new(payload.clone(), current_timestamp);
        
        self.memory.evaluate_and_store(new_pocket);

        // Return the delta so the caller (the eventual main.rs) can observe the learning process
        payload.delta
    }
}
