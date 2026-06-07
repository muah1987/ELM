//! Universal Experience Encoder (UEE)
//! This module handles the normalization and encoding of diverse sensor signals
//! into a uniform representation suitable for the Memory Bank.

pub struct Uee;

impl Uee {
    /// Normalizes a value to the range [0.0, 1.0] based on a known min and max.
    pub fn normalize(value: f32, min: f32, max: f32) -> f32 {
        if max <= min {
            return 0.0;
        }
        let clamped = value.clamp(min, max);
        (clamped - min) / (max - min)
    }

    /// Encodes raw focal state (position, touch) into a normalized vector.
    pub fn encode_focal(x: i32, y: i32, touch: bool, grid_size: i32) -> [f32; 4] {
        [
            Self::normalize(x as f32, 0.0, grid_size as f32),
            Self::normalize(y as f32, 0.0, grid_size as f32),
            if touch { 1.0 } else { 0.0 },
            0.0, // Reserved for future focal modal
        ]
    }

    /// Encodes ambient state (temperature) into a normalized vector.
    pub fn encode_ambient(temp: f32) -> f32 {
        // Assuming a range of -20C to 100C for ambient temperature
        Self::normalize(temp, -20.0, 100.0)
    }
}
