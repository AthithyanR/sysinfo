use axum::{
    debug_handler,
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::IntoResponse,
    routing::{get, get_service},
    serve, Router,
};
use std::thread;
use sysinfo::{CpuExt, System, SystemExt};
use tokio::sync::broadcast;
use tower_http::services::ServeDir;

type CpuState = Vec<f32>;

#[derive(Clone)]
struct AppState {
    tx: broadcast::Sender<CpuState>,
}

#[tokio::main]
async fn main() {
    let (tx, _) = broadcast::channel::<CpuState>(1);

    let app_state = AppState { tx: tx.clone() };

    let router = Router::new()
        .route("/realtime/cpus", get(realtime_cpus))
        .nest_service("/", get_service(ServeDir::new("static")))
        .with_state(app_state.clone());

    tokio::task::spawn_blocking(move || {
        let mut sys = System::new();
        loop {
            sys.refresh_cpu();
            let _ = tx.send(sys.cpus().iter().map(|cpu| cpu.cpu_usage()).collect());
            thread::sleep(System::MINIMUM_CPU_UPDATE_INTERVAL);
        }
    });

    let listener = tokio::net::TcpListener::bind("0.0.0.0:7111").await.unwrap();
    println!("Listening on {}", listener.local_addr().unwrap());
    serve(listener, router.into_make_service()).await.unwrap();
}

#[debug_handler]
async fn realtime_cpus(
    State(app_state): State<AppState>,
    wsup: WebSocketUpgrade,
) -> impl IntoResponse {
    wsup.on_upgrade(move |ws| async {
        stream_cpus(app_state, ws).await;
    })
}

async fn stream_cpus(app_state: AppState, mut ws: WebSocket) {
    let mut rx = app_state.tx.subscribe();

    while let Ok(cpus) = rx.recv().await {
        ws.send(Message::Text(serde_json::to_string(&cpus).unwrap()))
            .await
            .unwrap();
    }
}
