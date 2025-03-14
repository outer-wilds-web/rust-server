use serde::Serialize;

pub mod planet;
pub mod ship;
pub mod solar_system;

#[derive(Debug, Clone, Serialize)]
pub struct Body {
    pub mass: f64,
    pub position: (f64, f64, f64),
    pub speed: (f64, f64, f64),
    pub direction: (f64, f64, f64),
}

impl Body {
    pub fn new(
        mass: f64,
        position: (f64, f64, f64),
        speed: (f64, f64, f64),
        direction: (f64, f64, f64),
    ) -> Self {
        Self {
            mass,
            position,
            speed,
            direction,
        }
    }

    pub fn update(&mut self, delta_time: f64) {
        self.position.0 += self.speed.0 * delta_time;
        self.position.1 += self.speed.1 * delta_time;
        self.position.2 += self.speed.2 * delta_time;
    }

    pub fn apply_force(&mut self, force: (f64, f64, f64), delta_time: f64) {
        self.speed.0 -= force.0 * delta_time / self.mass;
        self.speed.1 -= force.1 * delta_time / self.mass;
        self.speed.2 -= force.2 * delta_time / self.mass;
    }

    pub fn gravitational_force(&self, other: &Body) -> (f64, f64, f64) {
        let g = 6.67430e-11; // Constante gravitationnelle
        let dx = other.position.0 - self.position.0;
        let dy = other.position.1 - self.position.1;
        let dz = other.position.2 - self.position.2;
        let distance = (dx * dx + dy * dy + dz * dz).sqrt();
        let force_magnitude = g * self.mass * other.mass / (distance * distance);
        (
            force_magnitude * dx / distance,
            force_magnitude * dy / distance,
            force_magnitude * dz / distance,
        )
    }

    pub fn distance(&self, other: &Body) -> f64 {
        let dx = other.position.0 - self.position.0;
        let dy = other.position.1 - self.position.1;
        let dz = other.position.2 - self.position.2;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
}
