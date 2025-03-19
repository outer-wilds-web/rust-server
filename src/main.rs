use dotenv::dotenv;
use serde::Serialize;
use solar_sytem_simulation::body::solar_system::SolarSystem;
use solar_sytem_simulation::server::Server;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::{env, thread};
use uuid::Uuid;
use warp::Filter;

#[derive(Serialize)]
struct ApiUrls {
    backend_url: String,
    websocket_url: String,
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    // Récupérer et afficher la variable d'environnement au démarrage
    let websocket_url =
        env::var("WEBSOCKET_URL").unwrap_or_else(|_| "ws://127.0.0.1:3012".to_string());
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

    let sleep_time = env::var("SIMULATION_SLEEP_TIME_MICROSECONDS")
        .unwrap_or_else(|_| "1000000".to_string())
        .parse::<u64>()
        .unwrap();

    // Thread to update the solar system
    thread::spawn(move || {
        loop {
            {
                let mut solar_system = solar_system_clone.lock().unwrap();
                solar_system.update(1.0 / 60.0);
            }

            // Vitesse du serveur
            thread::sleep(Duration::from_micros(sleep_time));
        }
    });

    let websocket_host = env::var("WEBSOCKET_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
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
