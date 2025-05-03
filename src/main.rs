use quantedge_x::api::create_router;
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
    axum::serve(listener, app).await.unwrap();
}
