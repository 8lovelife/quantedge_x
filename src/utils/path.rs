use dotenv::dotenv;
use std::env;
use std::path::PathBuf;

pub fn get_kv_db_path() -> PathBuf {
    dotenv().ok();

    let db_path = env::var("KV_DB_PATH").unwrap_or_else(|_| {
        let mut default_path = if let Ok(home) = env::var("HOME") {
            PathBuf::from(home)
        } else {
            PathBuf::from(".")
        };

        default_path.push(".quantedge");
        default_path.push("data");
        default_path.push("kv_data.db");

        default_path.to_str().unwrap().to_string()
    });

    let path = PathBuf::from(db_path);

    // Create directories if they don't exist
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("Failed to create database directory");
    }

    path
}

pub fn get_chart_db_path() -> PathBuf {
    dotenv().ok();

    let db_path = env::var("CHART_DB_PATH").unwrap_or_else(|_| {
        let mut default_path = if let Ok(home) = env::var("HOME") {
            PathBuf::from(home)
        } else {
            PathBuf::from(".")
        };

        default_path.push(".quantedge");
        default_path.push("data");
        default_path.push("chart_data.db");

        default_path.to_str().unwrap().to_string()
    });

    let path = PathBuf::from(db_path);

    // Create directories if they don't exist
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("Failed to create database directory");
    }

    path
}

pub fn get_db_path() -> PathBuf {
    dotenv().ok();

    let db_path = env::var("DB_PATH").unwrap_or_else(|_| {
        let mut default_path = if let Ok(home) = env::var("HOME") {
            PathBuf::from(home)
        } else {
            PathBuf::from(".")
        };

        default_path.push(".quantedge");
        default_path.push("data");
        default_path.push("market_data.db");

        default_path.to_str().unwrap().to_string()
    });

    let path = PathBuf::from(db_path);

    // Create directories if they don't exist
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("Failed to create database directory");
    }

    path
}

pub fn get_db_path_str() -> String {
    get_db_path().to_str().unwrap().to_string()
}

pub fn get_chart_db_path_str() -> String {
    get_chart_db_path().to_str().unwrap().to_string()
}

pub fn get_kv_db_path_str() -> String {
    get_kv_db_path().to_str().unwrap().to_string()
}

pub fn get_coin_api_key() -> String {
    dotenv().ok();
    env::var("COINAPI_KEY").unwrap().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_get_db_path_with_env() {
        // Create a temporary directory for testing
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().join("test.db");

        // Set DB_PATH environment variable
        unsafe {
            env::set_var("DB_PATH", temp_path.to_str().unwrap());
        }

        // Get the path
        let path = get_db_path();

        // Verify the path matches the environment variable
        assert_eq!(path, temp_path);

        // Verify the parent directory exists
        assert!(path.parent().unwrap().exists());
    }

    #[test]
    fn test_get_db_path_without_env() {
        // Remove DB_PATH environment variable if it exists
        unsafe {
            env::remove_var("DB_PATH");
        }

        // Get the path
        let path = get_db_path();

        // Verify the path is in the home directory
        let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
        assert!(path.to_str().unwrap().starts_with(&home));

        // Verify the path contains the expected components
        let path_str = path.to_str().unwrap();
        assert!(path_str.contains(".quantedge"));
        assert!(path_str.contains("data"));
        assert!(path_str.ends_with("trades.db"));

        // Verify the parent directory exists
        assert!(path.parent().unwrap().exists());
    }

    #[test]
    fn test_get_db_path_str() {
        // Create a temporary directory for testing
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().join("test.db");

        // Set DB_PATH environment variable
        unsafe {
            env::set_var("DB_PATH", temp_path.to_str().unwrap());
        }

        // Get the path string
        let path_str = get_db_path_str();

        // Verify the path string matches the environment variable
        assert_eq!(path_str, temp_path.to_str().unwrap());
    }

    #[test]
    fn test_directory_creation() {
        // Create a temporary directory for testing
        let temp_dir = tempdir().unwrap();
        let nested_path = temp_dir.path().join("nested").join("path").join("test.db");

        // Set DB_PATH environment variable
        unsafe {
            env::set_var("DB_PATH", nested_path.to_str().unwrap());
        }

        // Get the path (this should create the directories)
        let path = get_db_path();

        // Verify all parent directories exist
        let mut current = path.parent().unwrap();
        while let Some(parent) = current.parent() {
            assert!(parent.exists());
            current = parent;
        }
    }

    #[test]
    fn test_path_consistency() {
        // Get the path twice
        let path1 = get_db_path();
        let path2 = get_db_path();

        // Verify they are the same
        assert_eq!(path1, path2);

        // Get the path string twice
        let path_str1 = get_db_path_str();
        let path_str2 = get_db_path_str();

        // Verify they are the same
        assert_eq!(path_str1, path_str2);

        // Verify the string matches the PathBuf
        assert_eq!(path_str1, path1.to_str().unwrap());
    }
}
