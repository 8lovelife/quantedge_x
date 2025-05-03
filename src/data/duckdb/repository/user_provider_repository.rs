use std::{
    sync::{
        Arc, Mutex,
        mpsc::{self, Sender},
    },
    thread,
};

use duckdb::{Connection, params};

use crate::data::duckdb::{
    schema::user_provider::{Command, UserProvider},
    types::Timestamp,
};

pub struct UserProviderRepository {
    sender: Sender<Command>,
}

impl UserProviderRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            while let Ok(command) = rx.recv() {
                let conn = conn.lock().expect("Failed to lock Connection");
                match command {
                    Command::Insert {
                        user_provider,
                        respond_to,
                    } => {
                        let user_provider = Self::do_create(&conn, user_provider);
                        respond_to.send(user_provider).unwrap();
                    }
                    Command::Find {
                        provider,
                        provider_uid,
                        respond_to,
                    } => {
                        let user_provider =
                            Self::do_find_user_provider(&conn, provider, provider_uid);
                        respond_to.send(user_provider).unwrap();
                    }
                    Command::Update {
                        id,
                        access_token,
                        refresh_token,
                        expires_at,
                        respond_to,
                    } => {
                        let effect = Self::do_update_by_id(
                            &conn,
                            id,
                            access_token,
                            refresh_token,
                            expires_at,
                        );
                        respond_to.send(effect).unwrap();
                    }
                }
            }
        });

        Self { sender: tx }
    }

    fn do_create(conn: &Connection, user_provider: UserProvider) -> Option<UserProvider> {
        let mut stmt = conn
            .prepare(
                r#"INSERT INTO user_providers
                (user_id, provider, provider_uid, access_token, refresh_token, expires_at)
            VALUES (?, ?, ?, ?, ?, ?) RETURNING id"#,
            )
            .expect("Failed to prepare statement");

        let id = stmt
            .query_row(
                params![
                    user_provider.user_id,
                    user_provider.provider,
                    user_provider.provider_uid,
                    user_provider.access_token,
                    user_provider.refresh_token,
                    user_provider.expires_at.as_ref().map(|t| t.0.to_rfc3339()),
                ],
                |row| row.get(0),
            )
            .unwrap();
        Some(UserProvider {
            id: Some(id),
            ..user_provider
        })
    }

    fn do_find_user_provider(
        conn: &Connection,
        provider: String,
        provider_uid: String,
    ) -> Option<UserProvider> {
        let mut stmt = conn.prepare(
            "SELECT id, user_id, provider, provider_uid, access_token, refresh_token, expires_at, created_at, updated_at 
             FROM user_providers 
             WHERE provider = ? AND provider_uid = ?"
        ).expect("Failed to prepare statement");

        let mut rows = stmt
            .query(params![provider, provider_uid])
            .expect("Query failed");

        if let Some(row) = rows.next().expect("Failed to fetch row") {
            Some(UserProvider {
                id: row.get(0).ok(),
                user_id: row.get(1).expect("user_id missing"),
                provider: row.get(2).expect("provider missing"),
                provider_uid: row.get(3).expect("provider_uid missing"),
                access_token: row.get(4).expect("access_token missing"),
                refresh_token: row.get(5).expect("refresh_token missing"),
                expires_at: row.get(6).expect("expires_at missing"),
                created_at: row.get(7).expect("created_at missing"),
                updated_at: row.get(8).expect("updated_at missing"),
            })
        } else {
            None
        }
    }

    pub fn do_update_by_id(
        conn: &Connection,
        id: i64,
        access_token: String,
        refresh_token: String,
        expires_at: Timestamp,
    ) -> Option<bool> {
        let result = conn
            .execute(
                r#"
            UPDATE user_providers 
            SET access_token = ?, refresh_token = ?, expires_at = ?
            WHERE id = ?
            "#,
                [
                    &access_token,
                    &refresh_token,
                    &expires_at.0.to_rfc3339(),
                    &id.to_string(),
                ],
            )
            .unwrap();
        Some(result != 0)
    }

    pub fn create(&self, user_provider: UserProvider) -> Option<UserProvider> {
        let (tx, rx) = mpsc::channel();
        self.sender
            .send(Command::Insert {
                user_provider,
                respond_to: tx,
            })
            .unwrap();
        rx.recv().ok().flatten()
    }

    pub fn find_user_provider(
        &self,
        provider: String,
        provider_uid: String,
    ) -> Option<UserProvider> {
        let (tx, rx) = mpsc::channel();
        let cmd = Command::Find {
            provider,
            provider_uid,
            respond_to: tx,
        };
        self.sender.send(cmd).unwrap();
        rx.recv().ok().flatten()
    }

    pub fn update_user_provider(
        &self,
        id: i64,
        access_token: String,
        refresh_token: String,
        expires_at: Timestamp,
    ) -> Option<bool> {
        let (tx, rx) = mpsc::channel();
        let cmd = Command::Update {
            id,
            access_token,
            refresh_token,
            expires_at,
            respond_to: tx,
        };
        self.sender.send(cmd).unwrap();
        rx.recv().ok().flatten()
    }
}
