use std::{
    error::Error,
    sync::{Arc, Mutex},
};

use serde::{Serialize, de::DeserializeOwned};
use sled::Db;

use crate::{data::duckdb::repository::get_chart_db, utils::path::get_chart_db_path_str};

use super::user_db_manager::get_user_chart_manager;

pub struct ChartDB {
    db: Arc<Mutex<Db>>,
}

impl ChartDB {
    pub fn build(user_id: i64) -> Self {
        Self {
            db: get_user_chart_manager().get_chart_conn(user_id),
        }
    }
    /// Create a new instance of ChartDB by opening the database at the specified path.
    pub fn new() -> Result<Self, Box<dyn Error>> {
        // let db = sled::open(get_chart_db_path_str())?;
        Ok(Self { db: get_chart_db() })
    }

    pub fn retrieve_strategy_chart<T: DeserializeOwned>(
        &self,
        strategy_id: i64,
        run_id: i64,
    ) -> Result<Option<T>, Box<dyn Error>> {
        let chart_key = format!("instance-{strategy_id}-{run_id}");
        if let Some(ivec) = self.db.lock().unwrap().get(chart_key)? {
            let retrieved_str = std::str::from_utf8(&ivec)?;
            let data: T = serde_json::from_str(retrieved_str)?;
            Ok(Some(data))
        } else {
            Ok(None)
        }
    }

    pub fn store_strategy_chart<T: Serialize>(
        &self,
        strategy_id: i64,
        run_id: i64,
        data: &T,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let chart_key = format!("instance-{strategy_id}-{run_id}");
        let json_str = serde_json::to_string(data)?;
        self.db
            .lock()
            .unwrap()
            .insert(chart_key, json_str.as_bytes())?;
        self.db.lock().unwrap().flush()?;
        Ok(())
    }
    /// Store a JSON object under a given key.
    pub fn store_json<T: Serialize>(
        &self,
        key: &str,
        data: &T,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let json_str = serde_json::to_string(data)?;
        self.db.lock().unwrap().insert(key, json_str.as_bytes())?;
        self.db.lock().unwrap().flush()?;
        Ok(())
    }

    /// Retrieve a JSON object by key.
    pub fn retrieve<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, Box<dyn Error>> {
        if let Some(ivec) = self.db.lock().unwrap().get(key)? {
            let retrieved_str = std::str::from_utf8(&ivec)?;
            let data: T = serde_json::from_str(retrieved_str)?;
            Ok(Some(data))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use std::path::PathBuf;
    use tempfile::tempdir;

    // Helper function to create a temporary directory and return its path as a String.
    fn get_temp_db_path() -> String {
        // Create a temporary directory.
        let dir = tempdir().unwrap();
        // Create a database path within the temporary directory.
        let mut path: PathBuf = dir.path().to_path_buf();
        path.push("chart_db");
        // Convert the path to a string.
        path.to_str().unwrap().to_string()
    }

    // Override get_chart_db_path_str for tests.
    fn get_chart_db_path_str_for_test() -> String {
        get_temp_db_path()
    }

    // Create a new instance of ChartDB for testing.
    fn create_test_db() -> ChartDB {
        let db =
            sled::open(get_chart_db_path_str_for_test()).expect("Failed to open test database");
        ChartDB {
            db: Arc::new(Mutex::new(db)),
        }
    }

    #[test]
    fn test_store_and_retrieve_json_value() {
        let chart_db = create_test_db();

        // Test storing and retrieving a serde_json::Value.
        let json_data = serde_json::json!({
            "chart": "sales",
            "data": [100, 200, 150]
        });

        chart_db.store_json("chart1", &json_data).unwrap();
        let retrieved: Option<serde_json::Value> = chart_db.retrieve("chart1").unwrap();
        assert_eq!(retrieved, Some(json_data));
    }

    #[test]
    fn test_store_and_retrieve_custom_struct() {
        let chart_db = create_test_db();

        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct Person {
            name: String,
            age: u32,
        }

        let person = Person {
            name: "Alice".to_string(),
            age: 30,
        };

        chart_db.store_json("person1", &person).unwrap();
        let retrieved_person: Option<Person> = chart_db.retrieve("person1").unwrap();
        assert_eq!(retrieved_person, Some(person));
    }
}
