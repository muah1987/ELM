use crate::memory::pocket::{StateAmbient, StateFocal};

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

    /// Calculates ambient temperature: 20C base + heat source decay
    fn calculate_temperature(&self, x: i32, y: i32) -> f32 {
        let dx = (self.heat_source_x - x) as f32;
        let dy = (self.heat_source_y - y) as f32;
        let dist_sq = (dx * dx) + (dy * dy);
        20.0 + (50.0 / (1.0 + dist_sq))
    }

    fn is_touching_wall(&self, x: i32, y: i32) -> bool {
        x <= 0 || x >= self.width - 1 || y <= 0 || y >= self.height - 1
    }

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

    pub fn execute(&mut self, action: u8) -> StateFocal {
        let mut next_x = self.agent_x;
        let mut next_y = self.agent_y;

        match action {
            0 => next_y -= 1, // North
            1 => next_x += 1, // East
            2 => next_y += 1, // South
            3 => next_x -= 1, // West
            _ => {} 
        }

        let mut hit_wall = false;
        if next_x < 0 || next_x >= self.width || next_y < 0 || next_y >= self.height {
            hit_wall = true;
        } else {
            self.agent_x = next_x;
            self.agent_y = next_y;
        }

        StateFocal {
            position_x: self.agent_x,
            position_y: self.agent_y,
            touching_wall: hit_wall || self.is_touching_wall(self.agent_x, self.agent_y),
        }
    }
}
