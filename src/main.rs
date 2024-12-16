mod ship;

use serde_json::{self, json};
use ship::TheShip;
use std::f64::consts::PI;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use ws::{listen, Handler, Handshake, Message, Result, Sender};

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
    ship: Arc<Mutex<TheShip>>, // Ajout du vaisseau
}

impl Handler for Server {
    fn on_open(&mut self, _: Handshake) -> Result<()> {
        self.last_update = Instant::now();
        let solar_system_clone = Arc::clone(&self.solar_system);
        let last_update_clone = Arc::new(Mutex::new(self.last_update));
        let ship_clone = Arc::clone(&self.ship); // Utilisation du vaisseau existant
        let out_clone = self.out.clone();

        thread::spawn(move || {
            let mut last_update = last_update_clone.lock().unwrap();
            loop {
                let now = Instant::now();
                let delta_time = (now - *last_update).as_secs_f64();
                *last_update = now;

                {
                    let mut solar_system = solar_system_clone.lock().unwrap();
                    solar_system.update(delta_time);
                }
                {
                    let mut ship = ship_clone.lock().unwrap();
                    ship.update(delta_time);
                }

                // Envoyer les informations des planètes et du vaisseau via la websocket
                let positions = {
                    let solar_system = solar_system_clone.lock().unwrap();
                    solar_system.positions()
                };
                let ship_info = {
                    let ship = ship_clone.lock().unwrap();
                    ship.to_json()
                };
                let message = json!({
                    "planets": positions,
                    "ship": ship_info,
                });
                out_clone.send(Message::text(message.to_string())).unwrap();

                thread::sleep(Duration::from_millis(1000 / 60));
            }
        });

        Ok(())
    }

    fn on_message(&mut self, msg: Message) -> Result<()> {
        let msg_text = msg.into_text()?;
        if let Ok(data) = serde_json::from_str::<serde_json::Value>(&msg_text) {
            if let Some(data) = data.get("data") {
                if let Some(engines) = data.get("engines") {
                    let mut ship = self.ship.lock().unwrap();
                    ship.engines.front = engines.get("front").unwrap().as_bool().unwrap();
                    ship.engines.back = engines.get("back").unwrap().as_bool().unwrap();
                    ship.engines.left = engines.get("left").unwrap().as_bool().unwrap();
                    ship.engines.right = engines.get("right").unwrap().as_bool().unwrap();
                    ship.engines.up = engines.get("up").unwrap().as_bool().unwrap();
                    ship.engines.down = engines.get("down").unwrap().as_bool().unwrap();
                }

                if let Some(rotation) = data.get("rotation") {
                    let mut ship = self.ship.lock().unwrap();
                    ship.rotation_engines.left = rotation.get("left").unwrap().as_bool().unwrap();
                    ship.rotation_engines.right = rotation.get("right").unwrap().as_bool().unwrap();
                    ship.rotation_engines.up = rotation.get("up").unwrap().as_bool().unwrap();
                    ship.rotation_engines.down = rotation.get("down").unwrap().as_bool().unwrap();
                }
            }
        }
        Ok(())
    }
}

fn main() {
    listen("127.0.0.1:3012", |out| Server {
        out,
        solar_system: Arc::new(Mutex::new(SolarSystem::new())),
        last_update: Instant::now(),
        ship: Arc::new(Mutex::new(TheShip::new())),
    })
    .unwrap();
}
