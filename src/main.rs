mod kafka_producer;
mod ship;

use crate::kafka_producer::KafkaProducer;
use dotenv::dotenv;
use serde::Serialize;
use serde_json::{self, json};
use ship::TheShip;
use std::collections::HashMap;
use std::f64::consts::PI;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::{env, thread};
use uuid::Uuid;
use warp::Filter;
use ws::{Handler, Handshake, Message, Result, Sender};

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
    ships: HashMap<Uuid, Arc<Mutex<TheShip>>>,
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
            ships: HashMap::new(),
        }
    }

    fn update(&mut self, delta_time: f64) {
        for planet in &mut self.planets {
            planet.update_position(delta_time);
        }

        for ship in self.ships.values_mut() {
            ship.lock().unwrap().update(delta_time);
        }
    }

    fn add_ship(&mut self, ship: Arc<Mutex<TheShip>>) {
        let uuid = ship.lock().unwrap().uuid;
        self.ships.insert(uuid, ship);
    }

    fn remove_ship(&mut self, uuid: Uuid) {
        self.ships.remove(&uuid);
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
    ship_uuid: Uuid,
}

impl Handler for Server {
    fn on_open(&mut self, _: Handshake) -> Result<()> {
        println!("Websocket opened. Ship uuid {}", self.ship_uuid);
        self.last_update = Instant::now();
        let solar_system_clone = Arc::clone(&self.solar_system);
        let out_clone = self.out.clone();

        let ship = Arc::new(Mutex::new(TheShip::new()));
        let ship_clone = ship.clone();
        self.ship_uuid = ship.lock().unwrap().uuid;

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

                let ships: Vec<TheShip> = {
                    let solar_system = solar_system_clone.lock().unwrap();
                    solar_system
                        .ships
                        .values()
                        .into_iter()
                        .map(|ship| ship.lock().unwrap().clone())
                        .collect()
                };

                let ship_info = { ship_clone.lock().unwrap().to_json() };

                let message = json!({
                    "planets": positions,
                    "ship": ship_info,
                    "ships": ships,
                });
                out_clone.send(Message::text(message.to_string())).unwrap();

                thread::sleep(Duration::from_millis(1000/30))
            }
        });

        Ok(())
    }

    fn on_message(&mut self, msg: Message) -> Result<()> {
        let msg_text = msg.into_text()?;
        if let Ok(data) = serde_json::from_str::<serde_json::Value>(&msg_text) {
            if let Some(data) = data.get("data") {
                if let Some(engines) = data.get("engines") {
                    let solar_system = self.solar_system.lock().unwrap();
                    let ship = solar_system.ships.get(&self.ship_uuid).unwrap();
                    let mut ship = ship.lock().unwrap();
                    ship.engines.front = engines.get("front").unwrap().as_bool().unwrap();
                    ship.engines.back = engines.get("back").unwrap().as_bool().unwrap();
                    ship.engines.left = engines.get("left").unwrap().as_bool().unwrap();
                    ship.engines.right = engines.get("right").unwrap().as_bool().unwrap();
                    ship.engines.up = engines.get("up").unwrap().as_bool().unwrap();
                    ship.engines.down = engines.get("down").unwrap().as_bool().unwrap();
                }

                if let Some(rotation) = data.get("rotation") {
                    let solar_system = self.solar_system.lock().unwrap();
                    let ship = solar_system.ships.get(&self.ship_uuid).unwrap();
                    let mut ship = ship.lock().unwrap();
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

#[derive(Serialize)]
struct ApiUrls {
    backend_url: String,
    websocket_url: String,
}


#[tokio::main]
async fn main() {
    dotenv().ok();

    // Récupérer et afficher la variable d'environnement au démarrage
    let websocket_url = env::var("WEBSOCKET_URL").unwrap_or_else(|_| "ws://127.0.0.1:3012".to_string());
    println!("WEBSOCKET_URL: {}", websocket_url);

    let solar_system = Arc::new(Mutex::new(SolarSystem::new()));

    let auth_api_url = warp::path("auth-api-url").map(move || {
        let backend_url = env::var("BACKEND_URL").unwrap_or_else(|_| "URL not set".to_string());
        
        // La closure capture websocket_url si nécessaire
        let websocket_url = websocket_url.clone(); 

        let api_urls = ApiUrls {
            backend_url,
            websocket_url,
        };
        warp::reply::json(&api_urls)
    });

    let cors = warp::cors()
        .allow_any_origin()
        .allow_header("content-type")
        .allow_methods(["GET", "POST", "PUT", "DELETE", "OPTIONS"]);

    let routes = auth_api_url.with(cors);

    tokio::spawn(async move {
        warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
    });

    let solar_system_clone = Arc::clone(&solar_system);

    let kafka_brokers = env::var("KAFKA_BROKERS").unwrap_or_else(|_| "localhost:9092".to_string());
    let kafka_topic = env::var("KAFKA_TOPIC").unwrap_or_else(|_| "planet-positions".to_string());

    let kafka_producer = KafkaProducer::new(&kafka_brokers, &kafka_topic)
        .expect("Failed to create Kafka producer");


    let kafka_producer_clone = kafka_producer.clone();

    // Thread to update the solar system
    thread::spawn(move || {
        let mut last_update = Instant::now();

        loop {
            let now = Instant::now();
            let delta_time = (now - last_update).as_secs_f64();
            last_update = now;

            {
                let mut solar_system = solar_system_clone.lock().unwrap();
                solar_system.update(delta_time);
            }

            thread::sleep(Duration::from_millis(1000/30));
        }
    });

    let solar_system_clone = Arc::clone(&solar_system);

    // Thread to send position to Kafka (not the same frequency as the solar system update)
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(1));

        loop {
            interval.tick().await;

            let positions = {
                let solar_system = solar_system_clone.lock().unwrap();
                solar_system.positions()
            };

            if let Err(e) = kafka_producer_clone.send_planet_positions(positions).await {
                eprintln!("Failed to send positions to Kafka: {}", e);
            }

            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    });

    let websocket_host = env::var("WEBSOCKET_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let websocket_port = env::var("WEBSOCKET_PORT").unwrap_or_else(|_| "3012".to_string());

    let websocket_address = format!("{}:{}", websocket_host, websocket_port);
    println!("WebSocket server listening on {}", websocket_address);

    ws::listen(&websocket_address, |out| Server {
        out,
        solar_system: Arc::clone(&solar_system),
        last_update: Instant::now(),
        ship_uuid: Uuid::new_v4(),
    })
    .unwrap();
}
