use chrono::{DateTime, Duration, Utc};
use rand::{Rng, distributions::Alphanumeric};

use crate::{
    api::handlers::{AuthUserInfo, UserInfo, user_auth},
    data::duckdb::{
        repository::{UserProviderRepository, UserRepository, connection::get_db_conn},
        schema::{User, user_provider::UserProvider},
        types::Timestamp,
    },
};

pub struct UserAuthService {
    user_provider: UserProviderRepository,
    user: UserRepository,
}

impl UserAuthService {
    pub fn new() -> Self {
        let connection = get_db_conn();
        let user_provider = UserProviderRepository::new(connection.clone());
        let user = UserRepository::new(connection.clone());
        Self {
            user_provider,
            user,
        }
    }

    pub fn user_auth(&self, user_auth: AuthUserInfo) -> Option<UserInfo> {
        let provider_uid = user_auth.provider_uid.to_string();
        let provider = user_auth.provider.to_string();
        let user_provider = self.find_user_provider(provider, provider_uid);

        let user_info = match user_provider {
            Some(user_provider) => {
                let user_id = user_provider.user_id;
                self.user.update_by_id(user_id, user_auth.picture);
                let user_provider_id = user_provider.id.unwrap();

                let expires_in = user_auth.expires_in;
                let now: DateTime<Utc> = Utc::now();
                let later = now + Duration::seconds(expires_in);

                self.user_provider.update_user_provider(
                    user_provider_id,
                    user_auth.access_token,
                    user_auth.refresh_token,
                    Timestamp(later),
                );

                let user = self.user.find_user_by_id(user_id).unwrap();
                let token = rand::thread_rng()
                    .sample_iter(&Alphanumeric)
                    .take(32)
                    .map(char::from)
                    .collect();

                UserInfo {
                    id: user_provider.user_id,
                    name: user.name,
                    email: user.email,
                    avatar_url: user.avatar_url,
                    menus: vec![],
                    token: Some(token),
                    roles: user.roles.unwrap(),
                }
            }
            _ => self.create(user_auth).unwrap(),
        };

        Some(user_info)
    }

    pub fn create(&self, user_auth: AuthUserInfo) -> Option<UserInfo> {
        let user = User {
            id: None,
            email: user_auth.email,
            password_hash: None,
            name: user_auth.name,
            avatar_url: Some(user_auth.picture),
            is_active: None,
            created_at: None,
            updated_at: None,
            roles: None,
        };

        let user = self.user.create(user).unwrap();

        let expires_in = user_auth.expires_in;
        let now: DateTime<Utc> = Utc::now();
        let later = now + Duration::seconds(expires_in);
        let user_provider = UserProvider {
            id: None,
            user_id: user.id.unwrap(),
            provider: user_auth.provider,
            provider_uid: user_auth.provider_uid,
            access_token: user_auth.access_token,
            refresh_token: user_auth.refresh_token,
            expires_at: Some(Timestamp(later)),
            created_at: None,
            updated_at: None,
        };

        self.create_user_provider(user_provider).unwrap();

        let token = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();

        let user_info: UserInfo = UserInfo {
            id: user.id.unwrap(),
            name: user.name,
            email: user.email,
            avatar_url: user.avatar_url,
            menus: vec![],
            token: Some(token),
            roles: user.roles.unwrap(),
        };

        Some(user_info)
    }

    pub fn create_user(&self, user: User) -> Option<User> {
        self.user.create(user)
    }

    pub fn find_user(&self, email: String, password_hash: String) -> Option<User> {
        self.user.find_user(email, password_hash)
    }

    pub fn find_user_by_id(&self, id: i64) -> Option<User> {
        self.user.find_user_by_id(id)
    }

    pub fn create_user_provider(&self, user_provider: UserProvider) -> Option<UserProvider> {
        self.user_provider.create(user_provider)
    }

    pub fn find_user_provider(
        &self,
        provider: String,
        provider_uid: String,
    ) -> Option<UserProvider> {
        self.user_provider
            .find_user_provider(provider, provider_uid)
    }
}
