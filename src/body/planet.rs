use std::f64::consts::PI;

#[derive(Clone)]
pub struct Planet {
    pub name: String,
    pub distance_from_sun: f64,
    pub angle: f64,
    pub angular_velocity: f64, // radians per second
}

impl Planet {
    pub fn new(name: &str, distance_from_sun: f64, orbital_period: f64) -> Self {
        Self {
            name: name.to_string(),
            distance_from_sun,
            angle: 0.0,
            angular_velocity: 2.0 * PI / orbital_period,
        }
    }

    pub fn update_position(&mut self, delta_time: f64) {
        self.angle += self.angular_velocity * delta_time;
        if self.angle > 2.0 * PI {
            self.angle -= 2.0 * PI;
        }
    }

    pub fn position(&self) -> (f64, f64) {
        (
            self.distance_from_sun * self.angle.cos(),
            self.distance_from_sun * self.angle.sin(),
        )
    }
}
