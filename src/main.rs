use std::{sync::{Mutex, Arc}, thread};
use axum::{
    serve,
    Router,
    routing::{get, get_service},
    response::IntoResponse,
    extract::State,
    Json, debug_handler,
};
use sysinfo::{System, SystemExt, CpuExt};
use tower_http::services::ServeDir;

#[derive(Clone, Default, Debug)]
struct AppState {
    cpus: Arc<Mutex<Vec<f32>>>
}

#[tokio::main]
async fn main() {
    let app_state = AppState {
        cpus: Arc::new(Mutex::new(Vec::new()))
    };
        
    let router = Router::new()
        .route("/api/cpus", get(get_cpus))
        .nest_service("/", get_service(ServeDir ::new("static")))
        .with_state(app_state.clone());

    tokio::task::spawn_blocking(move || {
        let mut sys = System::new();
        loop {
            sys.refresh_cpu();
            {
                let mut cpus = app_state.cpus.lock().unwrap();
                *cpus = sys.cpus().iter().map(|cpu| cpu.cpu_usage()).collect();
            }
            thread::sleep(System::MINIMUM_CPU_UPDATE_INTERVAL);
        }
    });

    let listener = tokio::net::TcpListener::bind("0.0.0.0:7111").await.unwrap();
    println!("Listening on {}", listener.local_addr().unwrap());
    serve(listener, router.into_make_service()).await.unwrap();
}

#[debug_handler]
async fn get_cpus(State(app_state): State<AppState>) -> impl IntoResponse {
    let cpus = app_state.cpus.lock().unwrap();
    Json(cpus.clone())
}