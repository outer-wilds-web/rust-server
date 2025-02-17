use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use uuid::Uuid;

use super::planet::Planet;
use super::ship::TheShip;

#[derive(Clone)]
pub struct SolarSystem {
    pub planets: Vec<Planet>,
    pub ships: HashMap<Uuid, Arc<Mutex<TheShip>>>,
}

impl SolarSystem {
    pub fn new() -> Self {
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

    pub fn update(&mut self, delta_time: f64) {
        for planet in &mut self.planets {
            planet.update_position(delta_time);
        }

        for ship in self.ships.values_mut() {
            ship.lock().unwrap().update(delta_time);
        }
    }

    pub fn add_ship(&mut self, ship: Arc<Mutex<TheShip>>) {
        let uuid = ship.lock().unwrap().uuid;
        self.ships.insert(uuid, ship);
    }

    pub fn remove_ship(&mut self, uuid: Uuid) {
        self.ships.remove(&uuid);
    }

    pub fn positions(&self) -> Vec<(String, (f64, f64))> {
        self.planets
            .iter()
            .map(|p| (p.name.clone(), p.position()))
            .collect()
    }
}
