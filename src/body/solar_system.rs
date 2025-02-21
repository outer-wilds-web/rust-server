use std::collections::HashMap;
use uuid::Uuid;

use super::planet::Planet;
use super::ship::TheShip;

#[derive(Debug, Clone)]
pub struct SolarSystem {
    pub planets: Vec<Planet>,
    pub ships: HashMap<Uuid, TheShip>,
}

impl SolarSystem {
    pub fn new() -> Self {
        let sun_mass = 1.989e15;
        Self {
            planets: vec![
                Planet::new("Sun", (0.0, 0.0, 0.0), sun_mass, sun_mass),
                Planet::new("Mercury", (100.0, 0.0, 0.0), 3.285e3, sun_mass),
                Planet::new("Venus", (250.0, 100.0, 0.0), 4.867e4, sun_mass),
                Planet::new("Earth", (200.0, 300.0, 0.0), 5.972e4, sun_mass),
                Planet::new("Mars", (300.0, 500.0, 0.0), 6.39e3, sun_mass),
                Planet::new("Jupiter", (800.0, 600.0, 0.0), 1.898e7, sun_mass),
            ],
            ships: HashMap::new(),
        }
    }

    pub fn update(&mut self, delta_time: f64) {
        // Update planets
        for i in 0..self.planets.len() {
            for j in (i + 1)..self.planets.len() {
                let force = self.planets[i]
                    .body
                    .gravitational_force(&self.planets[j].body);
                self.planets[i].body.apply_force(force, delta_time);
                self.planets[j]
                    .body
                    .apply_force((-force.0, -force.1, -force.2), delta_time);
            }
        }

        // Update ships
        for ship in self.ships.values_mut() {
            for planet in &self.planets {
                let force = planet.body.gravitational_force(&ship.body);
                ship.body.apply_force(force, delta_time);
            }
        }

        self.planets.iter_mut().for_each(|p| p.update(delta_time));
        self.ships
            .iter_mut()
            .for_each(|(_, s)| s.update(delta_time));
    }

    pub fn add_ship(&mut self, ship: TheShip) {
        let uuid = ship.uuid;
        self.ships.insert(uuid, ship);
    }

    pub fn remove_ship(&mut self, uuid: Uuid) {
        self.ships.remove(&uuid);
    }

    pub fn positions(&self) -> Vec<(String, (f64, f64, f64))> {
        self.planets
            .iter()
            .map(|p| (p.name.clone(), p.body.position))
            .collect()
    }
}
