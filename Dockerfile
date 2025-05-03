# -------- Build Stage --------
    FROM rust:1.85.0 AS builder

    WORKDIR /app
    
    # Cache dependencies
    COPY Cargo.toml Cargo.lock ./
    RUN mkdir -p src && echo 'fn main() {}' > src/main.rs
    RUN cargo build --release --bin quantedge_x
    RUN rm -r src
    
    # Copy full project and rebuild
    COPY . .
    
    RUN cargo build --release --bin quantedge_x

    RUN cargo run --release --bin setup_db
    
    # -------- Runtime Stage --------
    FROM debian:bookworm-slim
    
    # For HTTPS requests (optional but often needed)
    RUN apt-get update && \
    apt-get install -y libssl3 ca-certificates && \
    rm -rf /var/lib/apt/lists/*
        
    WORKDIR /app
    COPY --from=builder /app/target/release/quantedge_x .
    COPY --from=builder /app/.quantedge_data.db .
    
    EXPOSE 3001
    
    CMD ["./quantedge_x"]