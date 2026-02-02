use std::str::FromStr;

use axum::{Json, extract::State, http::StatusCode};
use axum_extra::extract::cookie::CookieJar;

use rand::{Rng, distributions::Alphanumeric};
use serde::{Deserialize, Serialize};

use crate::{
    api::AppState,
    data::duckdb::{
        repository::connection::get_kv_db,
        schema::{User, user::Role},
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    Google,
    Github,
    Wechat,
}

impl FromStr for Provider {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "google" => Ok(Provider::Google),
            "github" => Ok(Provider::Github),
            "wechat" => Ok(Provider::Wechat),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthUserInfo {
    pub provider_uid: String,
    pub provider: String,
    pub name: String,
    pub picture: String,
    pub email: String,
    pub access_token: String,
    pub expires_in: i64,
    pub refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterResponse {
    user_info: Option<UserInfo>,
    user_exist: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserInfo {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub avatar_url: Option<String>,
    pub roles: Vec<Role>,
    pub menus: Vec<MenuItem>,
    pub token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuItem {
    pub id: i64,
    pub parent_id: Option<i64>,
    pub title: String,
    pub url: String,
    pub icon: Option<String>,
    pub order: Option<i32>,
    pub children: Option<Vec<MenuItem>>,
}

// export interface MenuItem {
//     title: string;
//     url: string;
//     icon?: string;
//     children?: MenuItem[];
// }

pub async fn user_register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Json<RegisterResponse>, StatusCode> {
    let email = payload.email;
    let password = payload.password;

    let user_auth_service = state.user_auth_service;
    let user = user_auth_service.find_user(email.to_string(), password.to_string());

    if user.is_none() {
        let user = User {
            name: email.to_string(),
            id: None,
            email,
            password_hash: Some(password.clone()),
            avatar_url: None,
            roles: None,
            is_active: None,
            created_at: None,
            updated_at: None,
        };

        let user = user_auth_service.create_user(user).unwrap();

        let mut user_info: UserInfo = UserInfo {
            id: user.id.unwrap(),
            name: user.name,
            email: user.email,
            avatar_url: user.avatar_url,
            menus: vec![],
            token: None,
            roles: user.roles.unwrap(),
        };

        let token: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();

        let kv_db = get_kv_db();
        let kv_db = kv_db.lock().unwrap();

        let json_str = serde_json::to_string(&user_info).unwrap();
        kv_db.insert(token.clone(), json_str.as_bytes()).unwrap();
        kv_db.flush().unwrap();

        let user_id = user_info.id;
        state.user_connection_manager.get_connection(user_id);

        user_info.token = Some(token);

        return Ok(Json(RegisterResponse {
            user_info: Some(user_info),
            user_exist: false,
        }));
    }

    Ok(Json(RegisterResponse {
        user_info: None,
        user_exist: true,
    }))
}

pub async fn user_login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<UserInfo>, StatusCode> {
    let email = payload.email;
    let password = payload.password;
    let user = state.user_auth_service.find_user(email, password);

    if let Some(user) = user {
        let mut user_info = UserInfo {
            id: user.id.unwrap(),
            name: user.name,
            email: user.email,
            roles: user.roles.unwrap(),
            menus: vec![],
            avatar_url: user.avatar_url,
            token: None,
        };

        let token: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();

        let kv_db = get_kv_db();
        let kv_db = kv_db.lock().unwrap();

        let json_str = serde_json::to_string(&user_info).unwrap();
        kv_db.insert(token.clone(), json_str.as_bytes()).unwrap();
        kv_db.flush().unwrap();

        let user_id = user_info.id;
        state.user_connection_manager.get_connection(user_id);

        user_info.token = Some(token);

        return Ok(Json(user_info));
    }

    Err(StatusCode::UNAUTHORIZED)
}

pub async fn user_auth(
    State(state): State<AppState>,
    Json(payload): Json<AuthUserInfo>,
) -> Result<Json<UserInfo>, StatusCode> {
    let user_info = state.user_auth_service.user_auth(payload).unwrap();
    let token = user_info
        .token
        .clone()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    let kv_db = get_kv_db();
    let kv_db = kv_db.lock().unwrap();

    let json_str = serde_json::to_string(&user_info).unwrap();
    kv_db.insert(token.clone(), json_str.as_bytes()).unwrap();
    kv_db.flush().unwrap();

    let user_id = user_info.id;
    state.user_connection_manager.get_connection(user_id);
    Ok(Json(user_info))
}

pub async fn get_current_user(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<Json<UserInfo>, StatusCode> {
    // let Some(session_cookie) = jar.get("session_id") else {
    //     return Err(StatusCode::UNAUTHORIZED);
    // };
    // let session_token = session_cookie.value();

    // let kv_db_arc = get_kv_db();
    // let kv_db = kv_db_arc.lock().unwrap();
    // let Some(user_info_bytes) = kv_db.get(session_token).unwrap() else {
    //     return Err(StatusCode::UNAUTHORIZED);
    // };
    // let user_info: UserInfo = serde_json::from_slice(&user_info_bytes).unwrap();

    let mut user_info = get_current_user_from_cookie(jar)?;

    let dashboard = MenuItem {
        id: 1,
        parent_id: None,
        title: "Dashboard".to_string(),
        url: "/dashboard".to_string(),
        icon: Some("LayoutDashboard".to_string()),
        order: Some(0),
        children: None,
    };

    let strategy = MenuItem {
        id: 2,
        parent_id: None,
        title: "Strategy".to_string(),
        url: "/strategies".to_string(),
        icon: Some("Library".to_string()),
        order: Some(1),
        children: None,
    };

    let lab = MenuItem {
        id: 3,
        parent_id: None,
        title: "Lab".to_string(),
        url: "/lab".to_string(),
        icon: Some("Gauge".to_string()),
        order: Some(2),
        children: None,
    };

    let exchange = MenuItem {
        id: 4,
        parent_id: None,
        title: "Exchange".to_string(),
        url: "/exchanges".to_string(),
        icon: Some("Database".to_string()),
        order: Some(3),
        children: None,
    };

    user_info.menus.push(exchange);
    user_info.menus.push(dashboard);
    user_info.menus.push(strategy);
    user_info.menus.push(lab);

    Ok(Json(user_info))
}

pub async fn revoke_current_user(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<(), StatusCode> {
    let Some(session_cookie) = jar.get("session_id") else {
        return Err(StatusCode::UNAUTHORIZED);
    };
    let session_token = session_cookie.value();
    let kv_db_arc = get_kv_db();
    let kv_db = kv_db_arc.lock().unwrap();
    kv_db.remove(session_token).unwrap();
    Ok(())
}

pub fn get_current_user_from_cookie(jar: CookieJar) -> Result<UserInfo, StatusCode> {
    let Some(session_cookie) = jar.get("session_id") else {
        return Err(StatusCode::UNAUTHORIZED);
    };
    let session_token = session_cookie.value();

    let kv_db_arc = get_kv_db();
    let kv_db = kv_db_arc.lock().unwrap();
    let Some(user_info_bytes) = kv_db.get(session_token).unwrap() else {
        return Err(StatusCode::UNAUTHORIZED);
    };
    let user_info: UserInfo = serde_json::from_slice(&user_info_bytes).unwrap();
    Ok(user_info)
}

pub fn get_current_user_from_token(token: String) -> Result<UserInfo, StatusCode> {
    let kv_db_arc = get_kv_db();
    let kv_db = kv_db_arc.lock().unwrap();
    let Some(user_info_bytes) = kv_db.get(token).unwrap() else {
        return Err(StatusCode::UNAUTHORIZED);
    };
    let user_info: UserInfo = serde_json::from_slice(&user_info_bytes).unwrap();
    Ok(user_info)
}
