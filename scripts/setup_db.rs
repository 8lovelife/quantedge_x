use chrono::{DateTime, Duration, Utc};
use dotenv::dotenv;
use duckdb::Connection;
use rand;
use serde::Deserialize;
use serde::Serialize;
use std::env;
use std::io;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug)]
enum SetupError {
    Io(io::Error),
    DuckDb(duckdb::Error),
}

impl From<io::Error> for SetupError {
    fn from(err: io::Error) -> Self {
        SetupError::Io(err)
    }
}

impl From<duckdb::Error> for SetupError {
    fn from(err: duckdb::Error) -> Self {
        SetupError::DuckDb(err)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Ohlcv {
    id: String,
    symbol: String,
    timestamp: DateTime<Utc>,
    open: String,
    high: String,
    low: String,
    close: String,
    volume: String,
}

#[derive(Debug)]
struct Trade {
    id: String,
    strategy: String,
    trade_type: String,
    asset: String,
    amount: String,
    price: String,
    timestamp: DateTime<Utc>,
    status: String,
    profit: Option<String>,
}

fn generate_mock_ohlcv_data(
    symbol: &str,
    base_price: f64,
    price_range: f64,
    num_points: i32,
    interval: &str,
) -> Vec<Ohlcv> {
    let mut data = Vec::new();
    let now = Utc::now();
    let duration = match interval {
        "hour" => Duration::hours(1),
        "day" => Duration::days(1),
        "week" => Duration::weeks(1),
        "month" => Duration::days(30),
        "year" => Duration::days(365),
        _ => Duration::hours(1),
    };

    for i in 0..num_points {
        let timestamp = now - duration * i;
        let open = base_price + (rand::random::<f64>() - 0.5) * price_range;
        let high = open * (1.0 + rand::random::<f64>() * 0.02);
        let low = open * (1.0 - rand::random::<f64>() * 0.02);
        let close = (high + low) / 2.0;
        let volume = rand::random::<f64>() * 1000.0;

        data.push(Ohlcv {
            id: format!("{}-{}-{}", symbol, interval, i),
            symbol: symbol.to_string(),
            timestamp,
            open: open.to_string(),
            high: high.to_string(),
            low: low.to_string(),
            close: close.to_string(),
            volume: volume.to_string(),
        });
    }

    data
}

fn generate_mock_trade_data() -> Vec<Trade> {
    let strategies = [
        "momentum",
        "mean_reversion",
        "breakout",
        "arbitrage",
        "trend_following",
    ];
    let trade_types = ["buy", "sell"];
    let assets = [
        "BTC", "ETH", "SOL", "DOGE", "XRP", "ADA", "LINK", "DOT", "AVAX", "MATIC",
    ];
    let statuses = ["completed", "pending", "cancelled", "failed"];

    let mut trades = Vec::new();
    let now = Utc::now();

    // Generate 50 mock trades
    for i in 0..50 {
        // Random strategy, type, asset, and status
        let strategy = strategies[rand::random::<usize>() % strategies.len()];
        let trade_type = trade_types[rand::random::<usize>() % trade_types.len()];
        let asset = assets[rand::random::<usize>() % assets.len()];
        let status = statuses[rand::random::<usize>() % statuses.len()];

        // Random amount between 0.01 and 2.0
        let amount = 0.01 + rand::random::<f64>() * 1.99;

        // Base price depends on the asset
        let base_price = match asset {
            "BTC" => 40000.0 + rand::random::<f64>() * 5000.0,
            "ETH" => 2000.0 + rand::random::<f64>() * 500.0,
            "SOL" => 90.0 + rand::random::<f64>() * 20.0,
            "DOGE" => 0.10 + rand::random::<f64>() * 0.05,
            "XRP" => 0.50 + rand::random::<f64>() * 0.10,
            "ADA" => 0.30 + rand::random::<f64>() * 0.10,
            "LINK" => 15.0 + rand::random::<f64>() * 5.0,
            "DOT" => 5.0 + rand::random::<f64>() * 2.0,
            "AVAX" => 30.0 + rand::random::<f64>() * 10.0,
            "MATIC" => 0.80 + rand::random::<f64>() * 0.20,
            _ => 100.0 + rand::random::<f64>() * 50.0,
        };

        // Random timestamp in the last 30 days
        let days_back = rand::random::<i64>() % 30;
        let hours_back = rand::random::<i64>() % 24;
        let timestamp = now - Duration::days(days_back) - Duration::hours(hours_back);

        // Profit only for completed trades (sometimes null)
        let profit = if status == "completed" && rand::random::<f64>() > 0.2 {
            let profit_value = if trade_type == "buy" {
                base_price * amount * (0.05 + rand::random::<f64>() * 0.10)
            } else {
                base_price * amount * (0.05 + rand::random::<f64>() * 0.10) * -1.0
            };
            Some(profit_value.to_string())
        } else {
            None
        };

        trades.push(Trade {
            id: Uuid::new_v4().to_string(),
            strategy: strategy.to_string(),
            trade_type: trade_type.to_string(),
            asset: asset.to_string(),
            amount: amount.to_string(),
            price: base_price.to_string(),
            timestamp,
            status: status.to_string(),
            profit,
        });
    }

    trades
}

fn insert_price_mock_data(conn: &Connection) -> Result<(), duckdb::Error> {
    let symbols = ["BTC", "ETH", "SOL"];
    let intervals = ["1h", "1d", "1w", "1m", "1y"];
    let base_prices = [42000.0, 2200.0, 100.0];
    let price_ranges = [500.0, 100.0, 10.0];

    for (i, symbol) in symbols.iter().enumerate() {
        for interval in intervals.iter() {
            let num_points = match *interval {
                "1h" => 6,
                "1d" => 7,
                "1w" => 6,
                "1m" => 3,
                "1y" => 5,
                _ => 6,
            };

            let data = generate_mock_ohlcv_data(
                symbol,
                base_prices[i],
                price_ranges[i],
                num_points,
                interval,
            );

            for ohlcv in data {
                conn.execute(
                    r#"
                    INSERT INTO ohlcv (id, symbol, timestamp, open, high, low, close, volume)
                    VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                    "#,
                    [
                        &ohlcv.id,
                        &ohlcv.symbol,
                        &ohlcv.timestamp.to_rfc3339(),
                        &ohlcv.open,
                        &ohlcv.high,
                        &ohlcv.low,
                        &ohlcv.close,
                        &ohlcv.volume,
                    ],
                )?;
            }
        }
    }

    Ok(())
}

fn insert_trade_mock_data(conn: &Connection) -> Result<(), duckdb::Error> {
    let trades = generate_mock_trade_data();

    for trade in trades {
        conn.execute(
            r#"
            INSERT INTO trades (id, strategy, type, asset, amount, price, timestamp, status, profit)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            [
                &trade.id,
                &trade.strategy,
                &trade.trade_type,
                &trade.asset,
                &trade.amount,
                &trade.price,
                &trade.timestamp.to_rfc3339(),
                &trade.status,
                &trade.profit.unwrap_or_default(),
            ],
        )?;
    }

    Ok(())
}

fn main() -> Result<(), SetupError> {
    // Load .env file
    dotenv().ok();

    let db_path = env::var("DB_PATH").unwrap_or_else(|_| ".quantedge_data.db".to_string());
    // let data_path = env::var("DATA_PATH").unwrap_or_else(|_| "ohlcv_data.parquet".to_string());
    let conn = Connection::open(&db_path)?;

    // Create trades table
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS trades (
            id TEXT PRIMARY KEY,
            strategy TEXT NOT NULL,
            type TEXT NOT NULL,
            asset TEXT NOT NULL,
            amount TEXT NOT NULL,
            price TEXT NOT NULL,
            timestamp TEXT NOT NULL,
            status TEXT NOT NULL,
            profit TEXT
        )
        "#,
        [],
    )?;

    // Create ohlcv table
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS ohlcv (
            id TEXT PRIMARY KEY,
            symbol TEXT NOT NULL,
            timestamp TEXT NOT NULL,
            open TEXT NOT NULL,
            high TEXT NOT NULL,
            low TEXT NOT NULL,
            close TEXT NOT NULL,
            volume TEXT NOT NULL,
            UNIQUE(symbol, timestamp)
        )
        "#,
        [],
    )?;

    // Create trade strategy table
    conn.execute(
        r#"CREATE SEQUENCE  IF NOT EXISTS trade_strategy_seq START 1"#,
        [],
    )?;

    conn.execute(
        r#"
            CREATE TABLE IF NOT EXISTS trade_strategy (
                id BIGINT PRIMARY KEY DEFAULT nextval('trade_strategy_seq'),
                name VARCHAR NOT NULL,
                description TEXT,
                algorithm VARCHAR NOT NULL,
                risk VARCHAR NOT NULL,
                allocation INTEGER NOT NULL,
                timeframe VARCHAR NOT NULL,
                assets VARCHAR NOT NULL,
                status VARCHAR NOT NULL,
                parameters JSON NOT NULL,
                latest_backtest_version BIGINT DEFAULT 0
            )
            "#,
        [],
    )?;

    // Create trade strategy_builds table
    conn.execute(
        r#"CREATE SEQUENCE  IF NOT EXISTS strategy_builds_seq START 1"#,
        [],
    )?;

    conn.execute(
        r#"
                CREATE TABLE IF NOT EXISTS strategy_builds (
                    id BIGINT PRIMARY KEY DEFAULT nextval('strategy_builds_seq'),
                    name VARCHAR NOT NULL,
                    description TEXT NOT NULL,
                    algorithm_type VARCHAR,
                    algorithm_sub_type VARCHAR,
                    assets JSON,
                    parameters JSON,
                    risk JSON,
                    progress VARCHAR CHECK(progress IN ('type', 'parameters', 'assets', 'risk', 'completed')) DEFAULT 'type',
                    lifecycle VARCHAR CHECK(lifecycle IN ('draft', 'backtest', 'paper', 'live','archived')) DEFAULT 'draft',
                    backtest_performance JSON,
                    paper_performance JSON,
                    live_performance JSON,
                    market_details JSON,
                    apply_backtest_version BIGINT DEFAULT 0,
                    latest_backtest_version BIGINT DEFAULT 0,
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    UNIQUE(name)
                )
                "#,
        [],
    )?;

    // {
    //     id: 1,
    //     name: "Mean Reversion",
    //     description: "Exploits price reversals to profit from short-term deviations from the average.",
    //     type: "mean-reversion",
    //     updated: new Date().toISOString(),
    //     backtestPerformance: {
    //         strategyReturn: 15.2,
    //         winRate: 65.0,
    //         maxDrawdown: -7.5
    //     },
    //     likes: 42,
    //     usageCount: 128,
    //     author: "tradingmaster"

    // "id": 1,
    // "value": "ma-crossover",
    // "label": "MA Crossover",
    // "desc": "Price returns to historical average",
    // "info": "MA Crossover strategy info",
    // "defaultParameters": {
    //   "fastPeriod": 10,
    //   "slowPeriod": 30,
    //   "subType": "sma",
    //   "positionType": "both",
    //   "rebalanceInterval": "daily",
    //   "entryThreshold": 1,
    //   "exitThreshold": 0.5
    // },
    // "defaultRisk": {
    //   "stopLoss": 0.05,
    //   "takeProfit": 0.1,
    //   "riskPerTrade": 0.02,
    //   "positionSize": 0.3,
    //   "maxConcurrentPositions": 1
    // },
    // "defaultExecution": {
    //   "slippage": 0.001,
    //   "commission": 0.0005,
    //   "entryDelay": 1,
    //   "minHoldingPeriod": 3,
    //   "maxHoldingPeriod": 10
    // }
    // },

    conn.execute(
        r#"CREATE SEQUENCE IF NOT EXISTS strategy_template_seq START 1"#,
        [],
    )?;

    conn.execute(
        r#"
                CREATE TABLE IF NOT EXISTS strategy_template (
                    id BIGINT PRIMARY KEY DEFAULT nextval('strategy_template_seq'),
                    name VARCHAR NOT NULL,
                    type VARCHAR NOT NULL,
                    description VARCHAR NOT NULL,
                    info TEXT NOT NULL,
                    parameters JSON,
                    risk JSON,
                    execution JSON,
                    performance JSON,
                    likes BIGINT DEFAULT 0,
                    usage BIGINT DEFAULT 0,
                    author VARCHAR DEFAULT 'system',
                    latest_lab_backtest_version BIGINT DEFAULT 0,
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    UNIQUE(name,type)
                )
                "#,
        [],
    )?;

    conn.execute(
        r#"CREATE SEQUENCE IF NOT EXISTS strategy_run_backtest_seq START 1"#,
        [],
    )?;

    conn.execute(
        r#"
            CREATE TABLE IF NOT EXISTS strategy_backtest_run_history (
                id BIGINT PRIMARY KEY DEFAULT nextval('strategy_run_backtest_seq'),
                strategy_id BIGINT NOT NULL,
                parameters JSON NOT NULL,
                performance JSON,
                market_details JSON,
                status     VARCHAR CHECK (status IN ('pending', 'running', 'success', 'failed')) NOT NULL,
                start_time TIMESTAMP NOT NULL,               
                end_time   TIMESTAMP, 
                created_at  TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
        "#, [])?;

    conn.execute(
        r#"CREATE SEQUENCE IF NOT EXISTS lab_run_backtest_seq START 1"#,
        [],
    )?;

    conn.execute(
            r#"
                CREATE TABLE IF NOT EXISTS lab_backtest_run_history (
                    id BIGINT PRIMARY KEY DEFAULT nextval('lab_run_backtest_seq'),
                    template_id BIGINT NOT NULL,
                    status     VARCHAR CHECK (status IN ('pending', 'running', 'success', 'failed')) NOT NULL,
                    parameters JSON NOT NULL,
                    performance JSON,
                    market_details JSON,
                    start_time TIMESTAMP NOT NULL,               
                    end_time   TIMESTAMP, 
                    created_at  TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                )
            "#, [])?;

    conn.execute(
        r#"CREATE SEQUENCE IF NOT EXISTS run_backtest_trade_seq START 1"#,
        [],
    )?;

    conn.execute(
        r#"
                CREATE TABLE IF NOT EXISTS backtest_run_trade (
                    id BIGINT PRIMARY KEY DEFAULT nextval('run_backtest_trade_seq'),
                    strategy_id BIGINT NOT NULL,
                    run_id BIGINT NOT NULL,
                    date VARCHAR NOT NULL,
                    trade_type VARCHAR CHECK (trade_type IN ('buy', 'sell')) NOT NULL,
                    result VARCHAR CHECK (result IN ('win', 'loss')) NOT NULL,
                    profit DOUBLE NOT NULL,
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                )
            "#,
        [],
    )?;

    conn.execute(
        r#"CREATE SEQUENCE IF NOT EXISTS run_backtest_balance_seq START 1"#,
        [],
    )?;

    conn.execute(
        r#"
                CREATE TABLE IF NOT EXISTS backtest_run_balance (
                    id BIGINT PRIMARY KEY DEFAULT nextval('run_backtest_balance_seq'),
                    strategy_id BIGINT NOT NULL,
                    run_id BIGINT NOT NULL,
                    date VARCHAR NOT NULL,
                    capital DOUBLE NOT NULL,
                    trades BIGINT NOT NULL,
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                )
            "#,
        [],
    )?;

    conn.execute(
        r#"CREATE SEQUENCE IF NOT EXISTS run_backtest_metrics_seq START 1"#,
        [],
    )?;

    conn.execute(
        r#"
                CREATE TABLE IF NOT EXISTS backtest_run_metrics (
                    id BIGINT PRIMARY KEY DEFAULT nextval('run_backtest_metrics_seq'),
                    strategy_id BIGINT NOT NULL,
                    run_id BIGINT NOT NULL,
                    date VARCHAR NOT NULL,
                    strategy_return DOUBLE NOT NULL,
                    max_drawdown DOUBLE NOT NULL,
                    win_rate DOUBLE NOT NULL,
                    sharpe_ratio DOUBLE NOT NULL,
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
                )
            "#,
        [],
    )?;

    conn.execute(
        r#"CREATE SEQUENCE IF NOT EXISTS user_providers_seq START 1"#,
        [],
    )?;

    conn.execute(
        r#"
                CREATE TABLE IF NOT EXISTS user_providers (
                    id BIGINT PRIMARY KEY DEFAULT nextval('user_providers_seq'),
                    user_id BIGINT NOT NULL,
                    provider VARCHAR(50) NOT NULL,            
                    provider_uid VARCHAR(255) NOT NULL,        
                    access_token TEXT,                         
                    refresh_token TEXT,                        
                    expires_at TIMESTAMP,                      
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    UNIQUE (provider, provider_uid)        
                )
            "#,
        [],
    )?;

    conn.execute(r#"CREATE SEQUENCE IF NOT EXISTS users_seq START 1"#, [])?;

    conn.execute(
        r#"
                CREATE TABLE IF NOT EXISTS users (
                    id BIGINT PRIMARY KEY DEFAULT nextval('users_seq'),
                    email VARCHAR(255) UNIQUE,            
                    password_hash VARCHAR(255),             
                    name VARCHAR(255),
                    avatar_url VARCHAR(512),
                    is_active BOOLEAN DEFAULT TRUE,
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP  
                )
            "#,
        [],
    )?;

    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS ohlcv_data (
            symbol VARCHAR NOT NULL,
            timestamp TIMESTAMP NOT NULL,
            open DOUBLE NOT NULL,
            high DOUBLE NOT NULL,
            low DOUBLE NOT NULL,
            close DOUBLE NOT NULL,
            volume DOUBLE NOT NULL,
            UNIQUE(symbol, timestamp)
        )
        "#,
        [],
    )?;

    let data_init_sql = &format!(
        "COPY ohlcv_data FROM '{}' (FORMAT PARQUET)",
        "ohlcv_data.parquet".to_string()
    );

    println!("data_init_sql ++ {:?}", data_init_sql);

    conn.execute_batch(data_init_sql)?;

    conn.execute(r#"CREATE SEQUENCE IF NOT EXISTS roles_seq START 1"#, [])?;

    conn.execute(
        r#"
                CREATE TABLE IF NOT EXISTS roles (
                    id BIGINT PRIMARY KEY DEFAULT nextval('roles_seq'),
                    name VARCHAR(255) UNIQUE,            
                    display_name VARCHAR(255) UNIQUE,  
                    description TEXT, 
                    is_default BOOLEAN DEFAULT FALSE,
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP  
                )
            "#,
        [],
    )?;

    let init_roles = r#"INSERT INTO roles (id, name, display_name, description, is_default) VALUES
    (1, 'admin', 'Administrator', 'Full access to all system functions', FALSE),
    (2, 'developer', 'Strategy Publisher', 'Can create and manage strategy templates', FALSE),
    (3, 'investor', 'Investor', 'Can deploy strategies and manage personal assets', TRUE);"#;

    conn.execute(init_roles, [])?;

    let init_algorithm = r#"INSERT INTO strategy_template (
    name, type, description, info,
    parameters, risk, execution, performance,
    likes, usage, author, latest_lab_backtest_version)
    VALUES
    (
        'MA Crossover',
        'ma-crossover',
        'A trend-following strategy that generates buy/sell signals based on the crossover of fast and slow moving averages.',
        'The Moving Average (MA) Crossover strategy enters a long position when the fast MA (e.g., 10-day) crosses above the slow MA (e.g., 30-day), and exits or goes short when the fast MA crosses below the slow MA. This approach helps capture trend reversals and is often used in both equity and crypto markets. Variants such as SMA, EMA, or WMA are supported to accommodate different smoothing preferences.',
        '{
            "options": { "meanType": ["sma", "ema", "wma"] },
            "default": {
                "meanType": "sma",
                "fastPeriod": 10,
                "slowPeriod": 30,
                "rebalanceInterval": "daily",
                "entryThreshold": 1.0,
                "exitThreshold": 0.5
            }
        }',
        '{
            "stopLoss": 5.0,
            "takeProfit": 10.0,
            "riskPerTrade": 2.0,
            "positionSize": 30.0,
            "maxConcurrentPositions": 1
        }',
        '{
            "slippage": 0.1,
            "commission": 0.05,
            "entryDelay": 1,
            "minHoldingPeriod": 3,
            "maxHoldingPeriod": 10
        }',
        NULL,
        0, 0, 'test', 0
    ),
    (
        'Mean Reversion',
        'mean-reversion',
        'A reversion-based strategy that assumes prices tend to revert to their historical mean over time.',
        'The Mean Reversion strategy identifies trading opportunities when asset prices deviate significantly from their historical average. It enters a long position when the price drops far below the mean, and a short position when it rises well above it...',
        '{
            "options": {
                "meanType": ["sma", "ema"],
                "reversionStyle": ["z-score", "bollinger"]
            },
            "default": {
                "lookbackPeriod": 20,
                "entryZScore": 2.0,
                "exitZScore": 0.5,
                "meanType": "ema",
                "reversionStyle": "z-score",
                "positionType": "both",
                "rebalanceInterval": "daily",
                "cooldownPeriod": 5,
                "bandMultiplier": 2
            }
        }',
        '{
            "stopLoss": 5.0,
            "takeProfit": 10.0,
            "riskPerTrade": 2.0,
            "positionSize": 30.0,
            "maxConcurrentPositions": 1
        }',
        '{
            "slippage": 0.1,
            "commission": 0.05,
            "entryDelay": 1,
            "minHoldingPeriod": 3,
            "maxHoldingPeriod": 10
        }',
        NULL,
        0, 0, 'system', 0
    );
    "#;

    conn.execute(init_algorithm, [])?;

    conn.execute(
        r#"CREATE SEQUENCE IF NOT EXISTS user_roles_seq START 1"#,
        [],
    )?;

    conn.execute(
        r#"
                CREATE TABLE IF NOT EXISTS user_roles (
                    id BIGINT PRIMARY KEY DEFAULT nextval('user_roles_seq'),
                    user_id BIGINT NOT NULL,
                    role_id BIGINT NOT NULL,
                    UNIQUE(user_id, role_id),
                    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP  
                )
            "#,
        [],
    )?;

    println!(
        "Database tables initialized and mock data inserted successfully at {}!",
        db_path
    );
    Ok(())
}
