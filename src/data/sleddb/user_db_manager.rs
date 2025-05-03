use std::{
    collections::HashMap,
    fs,
    path::Path,
    sync::{Arc, Mutex, OnceLock, RwLock},
};

use sled::Db;

static USER_CHART_DB_MANAGER: OnceLock<Arc<UserChartDbManager>> = OnceLock::new();

pub fn get_user_chart_manager() -> Arc<UserChartDbManager> {
    USER_CHART_DB_MANAGER
        .get_or_init(|| {
            let mannager = UserChartDbManager::new();
            Arc::new(mannager)
        })
        .clone()
}

pub struct UserChartDbManager {
    connections: RwLock<HashMap<i64, Arc<Mutex<Db>>>>,
}

impl UserChartDbManager {
    pub fn new() -> Self {
        Self {
            connections: RwLock::new(HashMap::new()),
        }
    }
    pub fn get_chart_conn(&self, user_id: i64) -> Arc<Mutex<Db>> {
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

        let dir_path = Path::new(".chart");
        if !dir_path.exists() {
            fs::create_dir_all(dir_path).expect("Failed to create .chart directory");
        }

        let path = format!(".chart/user_{}.db", user_id);
        let db = sled::open(&path).expect("Failed to open chart db!");
        let arc_db = Arc::new(Mutex::new(db));

        map.insert(user_id, arc_db.clone());
        arc_db
    }
}
