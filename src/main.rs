use quantedge_x::{api::create_router, ws::push_stream::start_ws_server};
use std::env;

#[tokio::main]
async fn main() {
    // Load environment variables from .env file
    dotenv::dotenv().ok();

    // Get server configuration from environment or use defaults
    let host = env::var("API_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = env::var("API_PORT").unwrap_or_else(|_| "3001".to_string());
    let addr = format!("{}:{}", host, port);
    // Create the router
    let app = create_router();

    // Start the server
    println!("Server running on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    let server = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    let ws_host = env::var("PUSH_STREAM_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let ws_port = env::var("PUSH_STREAM_PORT").unwrap_or_else(|_| "9001".to_string());
    let addr = format!("{}:{}", ws_host, ws_port);
    let ws_server = tokio::spawn(start_ws_server(addr));

    let _ = tokio::join!(server, ws_server);
}
