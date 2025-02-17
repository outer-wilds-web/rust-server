pub mod planet;
pub mod ship;
pub mod solar_system;

#[derive(Debug)]
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
}
