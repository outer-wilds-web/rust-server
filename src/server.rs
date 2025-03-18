use crate::body::ship::TheShip;
use crate::body::solar_system::SolarSystem;
use serde_json::{self, json};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use uuid::Uuid;
use ws::{Handler, Handshake, Message, Result, Sender};

pub struct Server {
    pub out: Sender,
    pub solar_system: Arc<Mutex<SolarSystem>>,
    pub last_update: Instant,
    pub ship_uuid: Uuid,
}

impl Handler for Server {
    fn on_open(&mut self, _: Handshake) -> Result<()> {
        println!("Websocket opened. Ship uuid {}", self.ship_uuid);
        self.last_update = Instant::now();
        let solar_system_clone = Arc::clone(&self.solar_system);
        let out_clone = self.out.clone();

        let ship = TheShip::new();
        self.ship_uuid = ship.uuid;
        let ship_uuid_clone = self.ship_uuid;

        {
            let mut solar_system = solar_system_clone.lock().unwrap();
            solar_system.add_ship(ship);
        }

        thread::spawn(move || {
            loop {
                // Envoyer les informations des planètes et du vaisseau via la websocket
                let positions = {
                    let solar_system = solar_system_clone.lock().unwrap();
                    solar_system.positions()
                };

                let planet_speeds = {
                    let solar_system = solar_system_clone.lock().unwrap();
                    solar_system.speeds()
                };

                let ships: Vec<TheShip> = {
                    let solar_system = solar_system_clone.lock().unwrap();
                    solar_system
                        .ships
                        .values()
                        .into_iter()
                        .map(|ship| ship.clone())
                        .collect()
                };

                let ship_info = {
                    let ship = ships.iter().find(|s| s.uuid == ship_uuid_clone).unwrap();
                    ship.to_json()
                };

                let message = json!({
                    "planets": positions,
                    "planet_speeds": planet_speeds,
                    "ship": ship_info,
                    "ships": ships,
                });
                out_clone.send(Message::text(message.to_string())).unwrap();

                // Vitesse d'envoi des informations via la websocket
                thread::sleep(Duration::from_micros(60))
                // thread::sleep(Duration::from_millis(1000))
            }
        });

        Ok(())
    }

    fn on_message(&mut self, msg: Message) -> Result<()> {
        let msg_text = msg.into_text()?;
        if let Ok(data) = serde_json::from_str::<serde_json::Value>(&msg_text) {
            if let Some(data) = data.get("data") {
                if let Some(engines) = data.get("engines") {
                    let mut solar_system = self.solar_system.lock().unwrap();
                    let ship = solar_system.ships.get_mut(&self.ship_uuid).unwrap();
                    ship.last_input = 0;
                    ship.engines.front = engines.get("front").unwrap().as_bool().unwrap();
                    ship.engines.back = engines.get("back").unwrap().as_bool().unwrap();
                    ship.engines.left = engines.get("left").unwrap().as_bool().unwrap();
                    ship.engines.right = engines.get("right").unwrap().as_bool().unwrap();
                    ship.engines.up = engines.get("up").unwrap().as_bool().unwrap();
                    ship.engines.down = engines.get("down").unwrap().as_bool().unwrap();
                }

                if let Some(rotation) = data.get("rotation") {
                    let mut solar_system = self.solar_system.lock().unwrap();
                    let ship = solar_system.ships.get_mut(&self.ship_uuid).unwrap();
                    ship.last_input = 0;
                    ship.rotation_engines.left = rotation.get("left").unwrap().as_bool().unwrap();
                    ship.rotation_engines.right = rotation.get("right").unwrap().as_bool().unwrap();
                    ship.rotation_engines.up = rotation.get("up").unwrap().as_bool().unwrap();
                    ship.rotation_engines.down = rotation.get("down").unwrap().as_bool().unwrap();
                }
            }
        }
        Ok(())
    }

    fn on_close(&mut self, code: ws::CloseCode, reason: &str) {
        let solar_system_clone = Arc::clone(&self.solar_system);
        {
            let mut solar_system = solar_system_clone.lock().unwrap();
            solar_system.remove_ship(self.ship_uuid);
        }
        println!("WebSocket closing for ({:?}) {}", code, reason);
    }
}
