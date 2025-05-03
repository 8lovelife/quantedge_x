use std::sync::mpsc::Sender;

use serde::{Deserialize, Serialize};

use crate::data::duckdb::types::Timestamp;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserProvider {
    pub id: Option<i64>,
    pub user_id: i64,
    pub provider: String,
    pub provider_uid: String,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: Option<Timestamp>,
    pub created_at: Option<Timestamp>,
    pub updated_at: Option<Timestamp>,
}

pub enum Command {
    Insert {
        user_provider: UserProvider,
        respond_to: Sender<Option<UserProvider>>,
    },
    Find {
        provider: String,
        provider_uid: String,
        respond_to: Sender<Option<UserProvider>>,
    },

    Update {
        id: i64,
        access_token: String,
        refresh_token: String,
        expires_at: Timestamp,
        respond_to: Sender<Option<bool>>,
    },
}
