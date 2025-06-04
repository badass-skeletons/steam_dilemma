use axum::{
    routing::get,
    Router,
};
use std::net::SocketAddr;
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Create the router
    let app = create_router();

    // Start the server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    
    println!("🚀 Server running on http://127.0.0.1:3000");
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    
    axum::serve(listener, app.layer(TraceLayer::new_for_http()))
        .await
        .unwrap();
}

fn create_router() -> Router {
    // Create a service to serve static files from the client/dist directory
    // with fallback to index.html for SPA routing
    let serve_dir = ServeDir::new("client/dist")
        .not_found_service(ServeFile::new("client/dist/index.html"));

    Router::new()
        // API routes can be added here in the future
        .route("/api/health", get(health_check))
        // Serve static files and SPA fallback
        .fallback_service(serve_dir)
}

async fn health_check() -> &'static str {
    "OK"
}