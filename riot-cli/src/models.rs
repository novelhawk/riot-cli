use chrono::{DateTime, Utc};

#[derive(Debug)]
pub struct AddUser {
    pub puuid: String,
    pub game_name: String,
    pub tag_line: String,
    pub region: String,
    pub user_info: String,
    pub entitlements_token: String,
    pub next_store: DateTime<Utc>,
    pub next_nightmarket: DateTime<Utc>,
}

#[derive(Debug)]
pub struct AddUserSession {
    pub user_id: i64,
    pub access_token: String,
    pub id_token: String,
    pub expires_at: DateTime<Utc>,
    pub authorized_cookies: String,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct User {
    pub id: i64,
    pub puuid: String,
    pub game_name: String,
    pub tag_line: String,
    pub region: String,
    pub user_info: String,
    pub entitlements_token: String,
    pub next_store: DateTime<Utc>,
    pub next_nightmarket: DateTime<Utc>,
    pub session: Option<UserSession>,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct UserSession {
    pub id: i64,
    pub user_id: i64,
    pub access_token: String,
    pub id_token: String,
    pub expires_at: DateTime<Utc>,
    pub authorized_cookies: String,
}
