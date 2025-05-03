use std::{
    sync::{
        Arc, Mutex,
        mpsc::{self, Sender},
    },
    thread,
};

use duckdb::{Connection, params};

use crate::data::duckdb::schema::{
    User,
    user::{Role, UserCommand, UserRole},
};

pub struct UserRepository {
    sender: Sender<UserCommand>,
}

impl UserRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            while let Ok(command) = rx.recv() {
                let conn = conn.lock().expect("Failed to lock Connection");
                match command {
                    UserCommand::Insert { user, respond_to } => {
                        let user_provider = Self::do_create(&conn, user);
                        respond_to.send(user_provider).unwrap();
                    }
                    UserCommand::Find {
                        email,
                        password_hash,
                        respond_to,
                    } => {
                        let user_provider = Self::do_find_user(&conn, email, password_hash);
                        respond_to.send(user_provider).unwrap();
                    }
                    UserCommand::FindById { id, respond_to } => {
                        let user_provider = Self::do_find_user_by_id(&conn, id);
                        respond_to.send(user_provider).unwrap();
                    }
                    UserCommand::Update {
                        id,
                        picture,
                        respond_to,
                    } => {
                        let user = Self::do_update_by_id(&conn, id, picture);
                        respond_to.send(user).unwrap();
                    }
                }
            }
        });

        Self { sender: tx }
    }

    fn do_create(conn: &Connection, user: User) -> Option<User> {
        let mut stmt = conn
            .prepare(
                r#"INSERT INTO users 
                (email, password_hash, name, avatar_url)
            VALUES (?, ?, ?, ?) RETURNING id"#,
            )
            .expect("Failed to prepare statement");

        let id = stmt
            .query_row(
                params![user.email, user.password_hash, user.name, user.avatar_url],
                |row| row.get(0),
            )
            .unwrap();

        let mut user = User {
            id: Some(id),
            ..user
        };

        let mut stmt = conn
        .prepare(r#"SELECT id,name,display_name,description FROM roles WHERE is_default = TRUE LIMIT 1"#)
        .expect("Failed to prepare statement");

        let mut rows = stmt.query([]).expect("Query failed");
        let role = if let Some(row) = rows.next().expect("Failed to fetch row") {
            Some(Role {
                id: row.get(0).ok()?,
                name: row.get(1).ok()?,
                display_name: row.get(2).ok()?,
                description: row.get(3).ok()?,
            })
        } else {
            None
        };

        if let Some(role) = role {
            conn.execute(
                "INSERT INTO user_roles (user_id, role_id) VALUES (?, ?)",
                params![user.id, role.id],
            )
            .unwrap();

            user.roles = Some(vec![role]);
        }
        Some(user)
    }

    fn do_find_user(conn: &Connection, email: String, password_hash: String) -> Option<User> {
        let mut stmt = conn.prepare(
            "SELECT id, email, password_hash, name, avatar_url, is_active, created_at, updated_at 
             FROM users 
             WHERE email = ? AND password_hash = ?"
        ).expect("Failed to prepare statement");

        let mut rows = stmt
            .query(params![email, password_hash])
            .expect("Query failed");

        let mut user = if let Some(row) = rows.next().expect("Failed to fetch row") {
            Some(User {
                id: row.get(0).ok(),
                email: row.get(1).expect("user_id missing"),
                password_hash: row.get(2).expect("password_hash missing"),
                name: row.get(3).expect("name missing"),
                avatar_url: row.get(4).expect("avatar_url missing"),
                is_active: row.get(5).expect("is_active missing"),
                created_at: row.get(6).expect("created_at missing"),
                updated_at: row.get(7).expect("updated_at missing"),
                roles: None,
            })
        } else {
            None
        };

        if user.is_none() {
            return None;
        }

        let user_id = user.as_ref().unwrap().id.unwrap();
        let roles = UserRepository::do_find_user_roles(conn, user_id);
        let mut user = user.unwrap();
        user.roles = Some(roles);
        Some(user)
    }

    fn do_find_user_by_id(conn: &Connection, id: i64) -> Option<User> {
        let mut stmt = conn.prepare(
            "SELECT id, email, password_hash, name, avatar_url, is_active, created_at, updated_at 
             FROM users 
             WHERE id = ?"
        ).expect("Failed to prepare statement");

        let mut rows = stmt.query(params![id]).expect("Query failed");

        let user = if let Some(row) = rows.next().expect("Failed to fetch row") {
            Some(User {
                id: row.get(0).ok(),
                email: row.get(1).expect("user_id missing"),
                password_hash: row.get(2).ok(),
                name: row.get(3).expect("name missing"),
                avatar_url: row.get(4).ok(),
                is_active: row.get(5).expect("is_active missing"),
                created_at: row.get(6).expect("created_at missing"),
                updated_at: row.get(7).expect("updated_at missing"),
                roles: None,
            })
        } else {
            None
        };

        if user.is_none() {
            return None;
        }

        let user_id = user.as_ref().unwrap().id.unwrap();
        let roles = UserRepository::do_find_user_roles(conn, user_id);
        let mut user = user.unwrap();
        user.roles = Some(roles);

        Some(user)
    }

    pub fn do_update_by_id(conn: &Connection, id: i64, picture: String) -> Option<bool> {
        let result = conn
            .execute(
                r#"
            UPDATE users 
            SET avatar_url = ?
            WHERE id = ?
            "#,
                [&picture, &id.to_string()],
            )
            .unwrap();
        Some(result != 0)
    }

    pub fn do_find_user_roles(conn: &Connection, user_id: i64) -> Vec<Role> {
        let mut stmt = conn
            .prepare(
                r#"
                SELECT r.id,r.name, r.display_name,r.description
                FROM user_roles ur
                JOIN roles r ON ur.role_id = r.id
                WHERE ur.user_id = ?
                "#,
            )
            .expect("Failed to prepare statement");

        let roles = stmt
            .query_map([&user_id], |row| {
                Ok(Role {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    display_name: row.get(2)?,
                    description: row.get(3)?,
                })
            })
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        roles
    }

    pub fn create(&self, user: User) -> Option<User> {
        let (tx, rx) = mpsc::channel();
        self.sender
            .send(UserCommand::Insert {
                user,
                respond_to: tx,
            })
            .unwrap();
        rx.recv().ok().flatten()
    }

    pub fn find_user(&self, email: String, password_hash: String) -> Option<User> {
        let (tx, rx) = mpsc::channel();
        let cmd = UserCommand::Find {
            email,
            password_hash,
            respond_to: tx,
        };
        self.sender.send(cmd).unwrap();
        rx.recv().ok().flatten()
    }

    pub fn find_user_by_id(&self, id: i64) -> Option<User> {
        let (tx, rx) = mpsc::channel();
        let cmd = UserCommand::FindById { id, respond_to: tx };
        self.sender.send(cmd).unwrap();
        rx.recv().ok().flatten()
    }

    pub fn update_by_id(&self, id: i64, picture: String) -> Option<bool> {
        let (tx, rx) = mpsc::channel();
        let cmd = UserCommand::Update {
            id,
            picture,
            respond_to: tx,
        };
        self.sender.send(cmd).unwrap();
        rx.recv().ok().flatten()
    }
}
