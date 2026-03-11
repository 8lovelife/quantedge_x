use quantedge_x::{
    api::create_router,
    data::market_data_bus::start_market_data_bus,
    matcher::{
        book::orderbook::OrderBook, engine::engine::Engine,
        policy::price_level::fifo::FifoPriceLevel, runtime::actor::BookActor,
        storage::localfile_storage::LocalFileStorage,
    },
    ws::push_stream::start_ws_server,
};
use std::env;

#[tokio::main]
async fn main() {
    // Load environment variables from .env file
    dotenv::dotenv().ok();

    let engine = Engine::start_with_publisher();
    let (client, _jh) = BookActor::<
        OrderBook<FifoPriceLevel, fn() -> FifoPriceLevel>, // T
        FifoPriceLevel,                                    // L
        fn() -> FifoPriceLevel,                            // F
        LocalFileStorage,                                  // S
    >::actor(1024, 300, engine);

    // Get server configuration from environment or use defaults
    let host = env::var("API_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = env::var("PORT").unwrap_or_else(|_| "3001".to_string());
    let addr = format!("{}:{}", host, port);
    // Create the router
    let app = create_router(client);

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
