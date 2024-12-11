use serde_json;
use std::f64::consts::PI;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use ws::{listen, CloseCode, Handler, Handshake, Message, Result, Sender};

#[derive(Clone)]
struct Planet {
    name: String,
    distance_from_sun: f64,
    angle: f64,
    angular_velocity: f64, // radians per second
}

impl Planet {
    fn new(name: &str, distance_from_sun: f64, orbital_period: f64) -> Self {
        Self {
            name: name.to_string(),
            distance_from_sun,
            angle: 0.0,
            angular_velocity: 2.0 * PI / orbital_period,
        }
    }

    fn update_position(&mut self, delta_time: f64) {
        self.angle += self.angular_velocity * delta_time;
        if self.angle > 2.0 * PI {
            self.angle -= 2.0 * PI;
        }
    }

    fn position(&self) -> (f64, f64) {
        (
            self.distance_from_sun * self.angle.cos(),
            self.distance_from_sun * self.angle.sin(),
        )
    }
}

#[derive(Clone)]
struct SolarSystem {
    planets: Vec<Planet>,
}

impl SolarSystem {
    fn new() -> Self {
        Self {
            planets: vec![
                Planet::new("Mercury", 50.0, 0.24 * 60.0),
                Planet::new("Venus", 70.0, 0.62 * 60.0),
                Planet::new("Earth", 90.0, 1.0 * 60.0),
                Planet::new("Mars", 110.0, 1.88 * 60.0),
                Planet::new("Jupiter", 150.0, 11.86 * 60.0),
            ],
        }
    }

    fn update(&mut self, delta_time: f64) {
        for planet in &mut self.planets {
            planet.update_position(delta_time);
        }
    }

    fn positions(&self) -> Vec<(String, (f64, f64))> {
        self.planets
            .iter()
            .map(|p| (p.name.clone(), p.position()))
            .collect()
    }
}

struct Server {
    out: Sender,
    solar_system: Arc<Mutex<SolarSystem>>,
    last_update: Instant,
}

impl Handler for Server {
    fn on_open(&mut self, _: Handshake) -> Result<()> {
        self.last_update = Instant::now();
        let out = self.out.clone();
        let solar_system = Arc::new(Mutex::new(self.solar_system.clone()));
        let last_update = Arc::new(Mutex::new(self.last_update));
        thread::spawn({
            let solar_system = Arc::clone(&solar_system);
            let last_update = Arc::clone(&last_update);
            move || loop {
                let now = Instant::now();
                let mut last_update = last_update.lock().unwrap();
                let delta_time = now.duration_since(*last_update).as_secs_f64();
                {
                    let solar_system = solar_system.lock().unwrap();
                    solar_system.lock().unwrap().update(delta_time);
                }
                *last_update = now;

                let positions = {
                    let solar_system = solar_system.lock().unwrap();
                    let positions = solar_system.lock().unwrap().positions();
                    positions
                };
                let positions_json = serde_json::to_string(&positions).unwrap();
                out.send(positions_json).unwrap();

                thread::sleep(Duration::from_millis(1000 / 60));
                // thread::sleep(Duration::from_millis(1000));
            }
        });
        Ok(())
    }

    fn on_message(&mut self, _: Message) -> Result<()> {
        Ok(())
    }

    fn on_close(&mut self, _: CloseCode, _: &str) {}
}

fn main() {
    listen("127.0.0.1:3012", |out| Server {
        out,
        solar_system: Arc::new(Mutex::new(SolarSystem::new())),
        last_update: Instant::now(),
    })
    .unwrap();
}
