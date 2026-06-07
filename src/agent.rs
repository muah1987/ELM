// The core cognitive loop. Executes one cycle of experience.

use crate::memory::pocket::{ExperiencePayload, Pocket};
use crate::memory::bank::MemoryBank;
use crate::sensors::state_self::StateSelf;
use crate::world::grid::GridWorld;
use crate::perception::uee::Uee;
use crate::planning::{Goal, engine::PlanningEngine};
use crate::hal::pu::{PuId, REGISTRY};

pub struct ELMAgent {
    pub env: GridWorld,
    pub memory: MemoryBank,
    pub goal: Goal,
    pub last_pain_level: f32,
}

impl ELMAgent {
    pub fn new() -> Self {
        Self {
            env: GridWorld::new(),
            memory: MemoryBank::new(),
            goal: Goal {
                target_focal: [0.8, 0.5, 0.0, 0.0], // High temperature target
                threshold: 0.05,
            },
            last_pain_level: 0.0,
        }
    }

    /// Reads the current hardware state.
    fn read_hardware_state(&self) -> StateSelf {
        StateSelf::read_native_with_pain().0
    }

    /// Dispatches a raw register write to a specific Processing Unit.
    /// This allows the agent to control hardware directly.
    pub fn dispatch_native_command(&self, pu_id: PuId, offset: usize, value: u32) {
        REGISTRY.write(pu_id, offset, value);
    }

    /// The World Model Query.
    fn predict_outcome(&mut self, payload: &ExperiencePayload) -> Option<[f32; 4]> {
        let mut best_sim = 0.0;
        let mut predicted_state: Option<[f32; 4]> = None;

        for (_, pocket) in self.memory.pockets.iter() {
            if pocket.compression_tier == 1 {
                let sim = self.memory.calculate_similarity(&pocket, &pocket);

                if sim > best_sim && sim > 0.90 && pocket.payload.action == payload.action {
                    best_sim = sim;
                    predicted_state = Some(pocket.payload.normalized_outcome.unwrap_or([0.0; 4]));
                }
            }
        }

        predicted_state
    }

    /// Calculates the hedonic valence of an experience based on pain, relief, and outcomes.
    fn calculate_valence(&mut self, pain: &crate::memory::pocket::PainSensor, outcome: &[f32; 4]) -> crate::memory::pocket::Valence {
        use crate::memory::pocket::{Valence, ValenceSource};

        // 1. Relief Detection (Transition from pain to non-pain)
        if self.last_pain_level > 0.5 && !pain.pain_active {
            self.last_pain_level = pain.pain_magnitude;
            return Valence {
                value: 0.5 + (self.last_pain_level * 0.5),
                source: ValenceSource::Relief,
                decay_modifier: 0.1,
            };
        }
        self.last_pain_level = pain.pain_magnitude;

        // 2. Active Pain
        if pain.pain_active {
            return Valence {
                value: -pain.pain_magnitude,
                source: ValenceSource::Pain,
                decay_modifier: 0.05,
            };
        }

        // 3. Reward (Alignment with Goal)
        let mut dist_to_goal = 0.0;
        for i in 0..4 {
            let diff = self.goal.target_focal[i] - outcome[i];
            dist_to_goal += diff * diff;
        }
        if dist_to_goal < self.goal.threshold {
            return Valence {
                value: 1.0,
                source: ValenceSource::Reward,
                decay_modifier: 0.3,
            };
        }

        // 4. Punishment (General negative outcome / Far from goal)
        if dist_to_goal > 0.8 {
            return Valence {
                value: -0.3,
                source: ValenceSource::Punishment,
                decay_modifier: 0.2,
            };
        }

        // Neutral
        Valence {
            value: 0.0,
            source: ValenceSource::Neutral,
            decay_modifier: 1.0,
        }
    }

    /// The core cognitive loop. Executes one cycle of experience.
    pub fn step(&mut self) -> f32 {
        let (initial_state, _) = StateSelf::read_native_with_pain();
        let cycle_start = initial_state.cpu_cycles;

        // 1. Capture Pre-Action States
        let (focal_state, ambient_state) = self.env.get_states();
        let self_state = self.read_hardware_state();

        // 2. Assemble Pre-Action Payload (Normalized via UEE)
        let mut payload = ExperiencePayload {
            normalized_self: [
                Uee::normalize(self_state.core_temp, 0.0, 100.0),
                Uee::normalize(self_state.cpu_cycles as f32, 0.0, 1e12),
                Uee::normalize(self_state.page_fault_count as f32, 0.0, 1000.0),
                0.0,
            ],
            normalized_ambient: [Uee::encode_ambient(ambient_state.grid_temp)],
            normalized_focal: Uee::encode_focal(focal_state.position_x, focal_state.position_y, focal_state.touching_wall, 10),
            action: 0,
            normalized_outcome: None,
            delta: 1.0,
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

        // 3. Plan the next action based on Goal
        let action = PlanningEngine::plan_next_action(&self.memory, &payload, &self.goal);
        payload.action = action;

        // 4. Predict the Future
        let predicted_outcome = self.predict_outcome(&payload);

        // 5. Execute Physical Action
        let actual_outcome = self.env.execute(action);
        payload.normalized_outcome = Some(Uee::encode_focal(actual_outcome.position_x, actual_outcome.position_y, actual_outcome.touching_wall, 10));

        // 6. Calculate Proprioceptive Latency
        let (final_state, _) = StateSelf::read_native_with_pain();
        let cycle_end = final_state.cpu_cycles;
        payload.normalized_self[3] = Uee::normalize((cycle_end - cycle_start) as f32, 0.0, 10000.0);

        // 7. Calculate Delta (Surprise)
        if let Some(predicted) = predicted_outcome {
            let mut dist_sq = 0.0;
            for i in 0..4 {
                let diff = predicted[i] - payload.normalized_outcome.unwrap()[i];
                dist_sq += diff * diff;
            }
            payload.delta = dist_sq / (1.0 + dist_sq);
        } else {
            payload.delta = 1.0;
        }

        // 8. Calculate Valence
        let current_pain = StateSelf::read_native_with_pain().1;
        let outcome_vec = payload.normalized_outcome.unwrap_or([0.0; 4]);
        payload.valence = self.calculate_valence(&current_pain, &outcome_vec);
        payload.pain_at_time = current_pain;

        // Hardware Pain Overrides
        if current_pain.pain_active {
            payload.delta = 1.0;
        }

        // 9. Store
        let new_pocket = Pocket::new(payload.clone(), cycle_end);
        self.memory.evaluate_and_store(new_pocket);

        payload.delta
    }
}
