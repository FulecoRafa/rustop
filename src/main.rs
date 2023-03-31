use axum::{
    extract::{ws::Message, State, WebSocketUpgrade},
    http::Response,
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use serde::Serialize;
use std::net::SocketAddr;
use sysinfo::{CpuExt, System, SystemExt};
use tokio::sync::broadcast;

#[derive(Serialize, Debug, Clone)]
struct ComputerData {
    cpus: Vec<f32>,  // Percentage of CPU usage
    ram: (u64, u64), // Percentage of RAM usage, total RAM
}

#[derive(Clone)]
struct AppState {
    data_chan: tokio::sync::broadcast::Sender<ComputerData>,
}

fn computer_poller(tx: tokio::sync::broadcast::Sender<ComputerData>) {
    let mut sys = sysinfo::System::new_all();

    loop {
        sys.refresh_cpu();
        sys.refresh_memory();

        let cpus: Vec<f32> = sys.cpus().iter().map(|cpu| cpu.cpu_usage()).collect();
        let ram = (sys.used_memory(), sys.total_memory());

        let data = ComputerData { cpus, ram };
        let _ = tx.send(data);

        std::thread::sleep(System::MINIMUM_CPU_UPDATE_INTERVAL);
    }
}

#[tokio::main]
async fn main() {
    let (tx, _) = broadcast::channel(1);

    let state = AppState {
        data_chan: tx.clone(),
    };

    let app = Router::new()
        .route("/", get(serve_html))
        .route("/index.css", get(serve_css))
        .route("/index.mjs", get(serve_mjs))
        .route("/ping", get(ping))
        .route("/sync", get(data_sync))
        .with_state(state);

    tokio::task::spawn_blocking(move || computer_poller(tx));

    let addr = SocketAddr::from(([0; 4], 6969));

    println!("Listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn ping() -> &'static str {
    "pong\n"
}

/// Serves static files
async fn serve_html() -> impl IntoResponse {
    let file = tokio::fs::read_to_string("static/index.html").await.unwrap();

    return Html(file).into_response();
}

async fn serve_css() -> impl IntoResponse {
    let file = tokio::fs::read_to_string("static/index.css").await.unwrap();


    Response::builder()
        .header("Content-Type", "text/css")
        .body(file)
        .unwrap()
        .into_response()
}

async fn serve_mjs() -> impl IntoResponse {
    let file = tokio::fs::read_to_string("static/index.mjs").await.unwrap();

    Response::builder()
        .header("Content-Type", "text/javascript")
        .body(file)
        .unwrap()
        .into_response()
}

/// Handles connection and evolves to a websocket connection
async fn data_sync(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}
async fn handle_socket(mut socket: axum::extract::ws::WebSocket, state: AppState) {
    let mut rx = state.data_chan.subscribe();
    while let Ok(msg) = rx.recv().await {
        let msg_str = serde_json::to_string(&msg).unwrap();
        socket.send(Message::Text(msg_str)).await.unwrap();
    }
}
