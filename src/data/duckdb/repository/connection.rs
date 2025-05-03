use std::{
    collections::HashMap,
    fs,
    path::Path,
    sync::{Arc, Mutex, OnceLock, RwLock},
};

use duckdb::Connection;
use sled::Db;

use crate::{
    data::duckdb::initialize_db::initialize_schema,
    utils::{
        get_db_path_str,
        path::{get_chart_db_path_str, get_kv_db_path_str},
    },
};

// pub fn get_connect(db_path: Option<String>) -> Result<Arc<Mutex<Connection>>, duckdb::Error> {
//     let db_path = db_path.unwrap_or_else(get_db_path_str);
//     let conn = Connection::open(&db_path)?;
//     Ok(Arc::new(Mutex::new(conn)))
// }

static USER_CONNECTION_MANAGER: OnceLock<Arc<UserConnectionManager>> = OnceLock::new();

static DB_CONN: OnceLock<Arc<Mutex<Connection>>> = OnceLock::new();

static CHART_DB: OnceLock<Arc<Mutex<Db>>> = OnceLock::new();

static KV_DB: OnceLock<Arc<Mutex<Db>>> = OnceLock::new();

pub fn get_db_conn() -> Arc<Mutex<Connection>> {
    DB_CONN
        .get_or_init(|| {
            let conn = Connection::open(get_db_path_str()).expect("Failed to open database!");
            Arc::new(Mutex::new(conn))
        })
        .clone()
}

pub fn get_chart_db() -> Arc<Mutex<Db>> {
    CHART_DB
        .get_or_init(|| {
            let db = sled::open(get_chart_db_path_str()).expect("Failed to open chart db!");
            Arc::new(Mutex::new(db))
        })
        .clone()
}

pub fn get_kv_db() -> Arc<Mutex<Db>> {
    KV_DB
        .get_or_init(|| {
            let db = sled::open(get_kv_db_path_str()).expect("Failed to open kv db!");
            Arc::new(Mutex::new(db))
        })
        .clone()
}

pub fn get_user_connection_manager() -> Arc<UserConnectionManager> {
    USER_CONNECTION_MANAGER
        .get_or_init(|| {
            let mannager = UserConnectionManager::new();
            Arc::new(mannager)
        })
        .clone()
}

pub struct UserConnectionManager {
    connections: RwLock<HashMap<i64, Arc<Mutex<Connection>>>>,
}

impl UserConnectionManager {
    pub fn new() -> Self {
        Self {
            connections: RwLock::new(HashMap::new()),
        }
    }
    pub fn get_connection(&self, user_id: i64) -> Arc<Mutex<Connection>> {
        {
            let map = self.connections.read().unwrap();
            if let Some(conn) = map.get(&user_id) {
                return conn.clone();
            }
        }
        let mut map = self.connections.write().unwrap();
        if let Some(conn) = map.get(&user_id) {
            return conn.clone();
        }

        let dir_path = Path::new(".data");
        if !dir_path.exists() {
            fs::create_dir_all(dir_path).expect("Failed to create .data directory");
        }

        let path = format!(".data/user_{}.db", user_id);
        let is_new = !Path::new(&path).exists();
        let conn = Connection::open(&path).expect("Failed to open DuckDB");

        let arc_conn = Arc::new(Mutex::new(conn));
        if is_new {
            initialize_schema(arc_conn.clone());
        }

        map.insert(user_id, arc_conn.clone());
        arc_conn
    }
}
