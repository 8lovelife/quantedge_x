use std::sync::mpsc::Sender;

use serde::{Deserialize, Serialize};

use crate::data::duckdb::types::Timestamp;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: Option<i64>,
    pub email: String,
    pub password_hash: Option<String>,
    pub name: String,
    pub avatar_url: Option<String>,
    pub roles: Option<Vec<Role>>,
    pub is_active: Option<bool>,
    pub created_at: Option<Timestamp>,
    pub updated_at: Option<Timestamp>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Role {
    pub id: i64,
    pub name: String,
    pub display_name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserRole {
    pub user_id: i64,
    pub role_id: i64,
}

pub enum UserCommand {
    Insert {
        user: User,
        respond_to: Sender<Option<User>>,
    },

    Find {
        email: String,
        password_hash: String,
        respond_to: Sender<Option<User>>,
    },

    FindById {
        id: i64,
        respond_to: Sender<Option<User>>,
    },

    Update {
        id: i64,
        picture: String,
        respond_to: Sender<Option<bool>>,
    },
}
