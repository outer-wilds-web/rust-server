use crate::utils::normalize_direction;

use super::Body;

#[derive(Debug, Clone)]
pub struct Planet {
    pub name: String,
    pub body: Body,
}

impl Planet {
    pub fn new(name: &str, initial_position: (f64, f64, f64), mass: f64, mass_sun: f64) -> Self {
        let initial_speed = Self::initial_speed(initial_position, mass_sun);
        Self {
            name: name.to_string(),
            body: Body::new(
                mass,
                initial_position,
                initial_speed,
                normalize_direction(initial_speed),
            ),
        }
    }

    pub fn update(&mut self, delta_time: f64) {
        self.body.update(delta_time);
    }

    pub fn initial_speed(initial_position: (f64, f64, f64), mass_sun: f64) -> (f64, f64, f64) {
        let g = 6.67430e-11; // Constante gravitationnelle
        let distance =
            (initial_position.0.powi(2) + initial_position.1.powi(2) + initial_position.2.powi(2))
                .sqrt();

        if distance == 0.0 {
            return (0.0, 0.0, 0.0);
        } else {
            let speed = (g * mass_sun / distance).sqrt();
            (
                -speed * initial_position.1 / distance,
                speed * initial_position.0 / distance,
                0.0,
            )
        }
    }
}
