#![no_std]

use crate::memory::{StateAmbient, StateFocal};

/// The physical universe for the MVP. A deterministic 2D matrix.
pub struct GridWorld {
    pub width: i32,
    pub height: i32,
    pub agent_x: i32,
    pub agent_y: i32,
    pub heat_source_x: i32,
    pub heat_source_y: i32,
}

impl GridWorld {
    /// Initializes the Grid World. 
    /// The agent starts in the top-left quadrant, with a heat source in the bottom-right.
    pub fn new() -> Self {
        Self {
            width: 10,
            height: 10,
            agent_x: 2, 
            agent_y: 2,
            heat_source_x: 8, 
            heat_source_y: 8,
        }
    }

    /// Calculates the ambient temperature based on the inverse square distance to the heat source.
    /// Uses pure floating-point math available in `core`.
    fn calculate_temperature(&self, x: i32, y: i32) -> f32 {
        let dx = (self.heat_source_x - x) as f32;
        let dy = (self.heat_source_y - y) as f32;
        let dist_sq = (dx * dx) + (dy * dy);
        
        // Base room temp is 20.0C. The source adds up to 50.0C, decaying over distance.
        20.0 + (50.0 / (1.0 + dist_sq))
    }

    /// Helper to determine if a coordinate is pressed against the environmental boundary.
    fn is_touching_wall(&self, x: i32, y: i32) -> bool {
        x <= 0 || x >= self.width - 1 || y <= 0 || y >= self.height - 1
    }

    /// Captures the observable state of the universe at the current moment.
    pub fn get_states(&self) -> (StateFocal, StateAmbient) {
        let focal = StateFocal {
            position_x: self.agent_x,
            position_y: self.agent_y,
            touching_wall: self.is_touching_wall(self.agent_x, self.agent_y),
        };

        let ambient = StateAmbient {
            grid_temp: self.calculate_temperature(self.agent_x, self.agent_y),
        };

        (focal, ambient)
    }

    /// Executes a physical action in the grid.
    /// Returns the resulting StateFocal to be compared against the ELM's prediction.
    pub fn execute(&mut self, action: u8) -> StateFocal {
        let mut next_x = self.agent_x;
        let mut next_y = self.agent_y;

        // 0: North, 1: East, 2: South, 3: West
        match action {
            0 => next_y -= 1,
            1 => next_x += 1,
            2 => next_y += 1,
            3 => next_x -= 1,
            _ => {} // Invalid action, state remains unchanged
        }

        // Rigid body physics simulation: Wall collisions
        let mut hit_wall = false;
        if next_x < 0 { 
            next_x = 0; 
            hit_wall = true; 
        }
        if next_x >= self.width { 
            next_x = self.width - 1; 
            hit_wall = true; 
        }
        if next_y < 0 { 
            next_y = 0; 
            hit_wall = true; 
        }
        if next_y >= self.height { 
            next_y = self.height - 1; 
            hit_wall = true; 
        }

        // Commit the movement
        self.agent_x = next_x;
        self.agent_y = next_y;

        // Return the *actual* outcome so the ELM can calculate its Delta
        StateFocal {
            position_x: self.agent_x,
            position_y: self.agent_y,
            touching_wall: hit_wall || self.is_touching_wall(self.agent_x, self.agent_y),
        }
    }
}
