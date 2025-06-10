mod steam;

use axum::{
    Json, Router,
    extract::State,
    response::Json as ResponseJson,
    routing::{get, post},
};
use library::{
    Consultant, CounterResponse, Customer, Game, NewCustomerResponse, Room, SteamGameLibrary,
};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use tokio::sync::RwLock;
use tower_http::{
    cors::CorsLayer,
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::steam::steam_client::SteamClient;

#[derive(Debug, Clone)]
pub struct AppModel {
    pub consultants: Vec<Consultant>,
    pub customers: Vec<Customer>,
    pub game_library: SteamGameLibrary,
    pub rooms: HashMap<u64, Room>,
    pub counter: u64,
}

impl AppModel {
    pub fn new() -> Self {
        Self {
            consultants: Vec::new(),
            customers: Vec::new(),
            game_library: SteamGameLibrary::new(),
            rooms: HashMap::new(),
            counter: 0,
        }
    }

    pub fn increment_counter(&mut self) -> u64 {
        self.counter += 1;
        self.counter
    }
}

// Server state containing the app model and other server-specific data
#[derive(Clone)]
struct AppState {
    app_model: Arc<RwLock<AppModel>>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Create the shared state
    let state = AppState {
        app_model: Arc::new(RwLock::new(AppModel::new())),
    };

    let app = create_router(state);

    // Start the server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    println!("🚀 Server running on http://127.0.0.1:3000");
    tracing::info!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app.layer(TraceLayer::new_for_http()))
        .await
        .unwrap();
}

fn create_router(state: AppState) -> Router {
    // Create a service to serve static files from the client/dist directory
    // with fallback to index.html for SPA routing
    let serve_dir =
        ServeDir::new("client/dist").not_found_service(ServeFile::new("client/dist/index.html"));

    // Configure CORS to allow requests from the client
    let cors = CorsLayer::permissive(); // This allows all origins, methods, and headers for development

    Router::new()
        // API routes
        .route("/api/health", get(health_check))
        .route("/api/increment", post(increment_counter))
        .route("/api/get_customer_library", post(get_customer_game_library))
        .layer(cors) // Add CORS layer to API routes
        // Serve static files and SPA fallback
        .fallback_service(serve_dir)
        .with_state(state)
}

async fn health_check() -> &'static str {
    "OK"
}

async fn increment_counter(State(state): State<AppState>) -> ResponseJson<CounterResponse> {
    let new_value = {
        let mut app_model = state.app_model.write().await;
        app_model.increment_counter()
    };
    tracing::info!("Counter incremented to: {}", new_value);

    ResponseJson(CounterResponse {
        counter_value: new_value,
    })
}

async fn get_customer_library_from_steam(steam_id: String) -> Customer {
    let steam_client = SteamClient::from("B72EE916D1F9D8B67E1D5C55AD6436F4".to_string());

    let mut customer = Customer {
        steam_name: "ass".to_owned(),
        steam_id: Some(123),
        games: Vec::new(),
    };

    match steam_client.get_user_library(&steam_id).await {
        Ok(library) => {
            log::info!("Library received with {} games", library.game_count);

            customer.games = library
                .games
                .into_iter()
                .map(|raw| Game::from(raw))
                .collect();
        }
        Err(error) => {
            log::error!("Can't get steam library : {error}")
        }
    }

    customer
}

async fn get_customer_game_library(
    State(state): State<AppState>,
    Json(steam_id_str): Json<String>,
) -> ResponseJson<NewCustomerResponse> {
    tracing::info!("Steam ID request: {}", steam_id_str);

    let customer = get_customer_library_from_steam(steam_id_str).await;

    ResponseJson(NewCustomerResponse { customer })
}
