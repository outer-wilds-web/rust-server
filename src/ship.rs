use serde::Serialize;
use serde_json::json;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct Engines {
    pub power: f64,
    pub front: bool,
    pub back: bool,
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct RotationEngines {
    pub power: f64,
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct TheShip {
    pub uuid: Uuid,
    pub speed: (f64, f64, f64),
    pub position: (f64, f64, f64),
    pub direction: (f64, f64, f64),
    pub engines: Engines,
    pub rotation_engines: RotationEngines,
    pub angle: f64,
    pub pitch: f64,
}

impl TheShip {
    pub fn new() -> Self {
        Self {
            uuid: Uuid::new_v4(),
            speed: (0.0, 0.0, 0.0),
            position: (0.0, 0.0, 450.0),
            direction: (1.0, 0.0, 0.0), // Always normalized
            engines: Engines {
                power: 1.0,
                front: false,
                back: false,
                left: false,
                right: false,
                up: false,
                down: false,
            },
            rotation_engines: RotationEngines {
                power: 0.5,
                left: false,
                right: false,
                up: false,
                down: false,
            },
            angle: -std::f64::consts::FRAC_PI_2,
            pitch: 0.0,
        }
    }

    pub fn to_json(&self) -> serde_json::Value {
        json!({
            "uuid": self.uuid.to_string(),
            "speed": self.speed,
            "position": self.position,
            "direction": self.direction,
            "engines": {
                "power": self.engines.power,
                "front": self.engines.front,
                "back": self.engines.back,
                "left": self.engines.left,
                "right": self.engines.right,
                "up": self.engines.up,
                "down": self.engines.down,
            },
            "rotation_engines": {
                "power": self.rotation_engines.power,
                "left": self.rotation_engines.left,
                "right": self.rotation_engines.right,
                "up": self.rotation_engines.up,
                "down": self.rotation_engines.down,
            },
        })
    }

    pub fn update(&mut self, delta_time: f64) {
        // Update the direction
        self.rotate(delta_time);

        // Update the speed
        self.accelerate(delta_time);

        // Update the position
        self.position.0 += self.speed.0 * delta_time;
        self.position.1 += self.speed.1 * delta_time;
        self.position.2 += self.speed.2 * delta_time;
        
        // Print the position to debug
        println!(
            "Updated speed: ({:.2}, {:.2}, {:.2})",
            self.speed.0, self.speed.1, self.speed.2
        );

        // Print the position to debug
        println!(
            "Updated position: ({:.2}, {:.2}, {:.2})",
            self.position.0, self.position.1, self.position.2
        );
    }

    pub fn accelerate(&mut self, delta_time: f64) {
        if self.engines.front {
            self.speed.0 -= self.direction.0 * delta_time * self.engines.power;
            self.speed.1 -= self.direction.1 * delta_time * self.engines.power;
            self.speed.2 -= self.direction.2 * delta_time * self.engines.power;
        }

        if self.engines.back {
            self.speed.0 += self.direction.0 * delta_time * self.engines.power;
            self.speed.1 += self.direction.1 * delta_time * self.engines.power;
            self.speed.2 += self.direction.2 * delta_time * self.engines.power;
        }

        // Vertical local direction
        let vertical_local = (
            -self.direction.0 * self.pitch.sin(),
            self.pitch.cos(),
            -self.direction.2 * self.pitch.sin(),
        );

        // Up vertical acceleration
        if self.engines.up {
            self.speed.0 -= vertical_local.0 * delta_time * self.engines.power;
            self.speed.1 -= vertical_local.1 * delta_time * self.engines.power;
            self.speed.2 -= vertical_local.2 * delta_time * self.engines.power;
        }

        // Down vertical acceleration
        if self.engines.down {
            self.speed.0 += vertical_local.0 * delta_time * self.engines.power;
            self.speed.1 += vertical_local.1 * delta_time * self.engines.power;
            self.speed.2 += vertical_local.2 * delta_time * self.engines.power;
        }

        // Lateral local direction
        let lateral_local = (
            self.direction.1 * 0.0 - self.direction.2 * 1.0,
            self.direction.2 * 0.0 - self.direction.0 * 0.0,
            self.direction.0 * 1.0 - self.direction.1 * 0.0,
        );

        // Left lateral acceleration
        if self.engines.left {
            self.speed.0 += lateral_local.0 * delta_time * self.engines.power;
            self.speed.1 += lateral_local.1 * delta_time * self.engines.power;
            self.speed.2 += lateral_local.2 * delta_time * self.engines.power;
        }

        // Right lateral acceleration
        if self.engines.right {
            self.speed.0 -= lateral_local.0 * delta_time * self.engines.power;
            self.speed.1 -= lateral_local.1 * delta_time * self.engines.power;
            self.speed.2 -= lateral_local.2 * delta_time * self.engines.power;
        }
    }

    /// Rotate the ship
    /// The ship is always normalized
    /// Values between -1.0 and 1.0
    pub fn rotate(&mut self, delta_time: f64) {
        let rotation_speed = delta_time * self.rotation_engines.power;

        if self.rotation_engines.left {
            self.angle += rotation_speed;
        }

        if self.rotation_engines.right {
            self.angle -= rotation_speed;
        }

        if self.rotation_engines.up {
            self.pitch -= rotation_speed;
        }

        if self.rotation_engines.down {
            self.pitch += rotation_speed;
        }

        // Update direction based on angle and pitch
        self.direction.0 = self.angle.cos() * self.pitch.cos();
        self.direction.1 = self.pitch.sin();
        self.direction.2 = self.angle.sin() * self.pitch.cos();

        // Normalize the direction
        let norm =
            (self.direction.0.powi(2) + self.direction.1.powi(2) + self.direction.2.powi(2)).sqrt();
        self.direction.0 /= norm;
        self.direction.1 /= norm;
        self.direction.2 /= norm;
    }
}
