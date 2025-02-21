use serde::Serialize;
use serde_json::json;
use uuid::Uuid;

use super::Body;

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
    pub body: Body,
    pub engines: Engines,
    pub rotation_engines: RotationEngines,
    pub angle: f64,
    pub pitch: f64,
}

impl TheShip {
    pub fn new() -> Self {
        Self {
            uuid: Uuid::new_v4(),
            body: Body::new(1.0, (300.0, 0.0, 0.0), (0.0, 0.0, 0.0), (0.0, 0.0, 0.0)),
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
            "speed": self.body.speed,
            "position": self.body.position,
            "direction": self.body.direction,
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
        if self.engines.back {
            self.body.apply_force(
                (
                    self.body.direction.0 * self.engines.power,
                    self.body.direction.1 * self.engines.power,
                    self.body.direction.2 * self.engines.power,
                ),
                delta_time,
            );
        }

        if self.engines.front {
            self.body.apply_force(
                (
                    -self.body.direction.0 * self.engines.power,
                    -self.body.direction.1 * self.engines.power,
                    -self.body.direction.2 * self.engines.power,
                ),
                delta_time,
            );
        }

        let vertical_local = (
            -self.body.direction.0 * self.pitch.sin(),
            self.pitch.cos(),
            -self.body.direction.2 * self.pitch.sin(),
        );

        // Up vertical acceleration
        if self.engines.up {
            self.body.apply_force(
                (
                    -vertical_local.0 * self.engines.power,
                    -vertical_local.1 * self.engines.power,
                    -vertical_local.2 * self.engines.power,
                ),
                delta_time,
            );
        }

        // Down vertical acceleration
        if self.engines.down {
            self.body.apply_force(
                (
                    vertical_local.0 * self.engines.power,
                    vertical_local.1 * self.engines.power,
                    vertical_local.2 * self.engines.power,
                ),
                delta_time,
            );
        }

        // Lateral local direction
        let lateral_local = (
            self.body.direction.1 * 0.0 - self.body.direction.2 * 1.0,
            self.body.direction.2 * 0.0 - self.body.direction.0 * 0.0,
            self.body.direction.0 * 1.0 - self.body.direction.1 * 0.0,
        );

        // Left lateral acceleration
        if self.engines.left {
            self.body.apply_force(
                (
                    lateral_local.0 * self.engines.power,
                    lateral_local.1 * self.engines.power,
                    lateral_local.2 * self.engines.power,
                ),
                delta_time,
            );
        }

        // Right lateral acceleration
        if self.engines.right {
            self.body.apply_force(
                (
                    -lateral_local.0 * self.engines.power,
                    -lateral_local.1 * self.engines.power,
                    -lateral_local.2 * self.engines.power,
                ),
                delta_time,
            );
        }

        self.rotate(delta_time);
        self.body.update(delta_time);
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
        self.body.direction.0 = self.angle.cos() * self.pitch.cos();
        self.body.direction.1 = self.pitch.sin();
        self.body.direction.2 = self.angle.sin() * self.pitch.cos();

        // Normalize the direction
        let norm = (self.body.direction.0.powi(2)
            + self.body.direction.1.powi(2)
            + self.body.direction.2.powi(2))
        .sqrt();
        self.body.direction.0 /= norm;
        self.body.direction.1 /= norm;
        self.body.direction.2 /= norm;
    }
}
