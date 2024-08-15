use chrono::{DateTime, Utc};

#[derive(Debug)]
pub struct User {
    pub id: i64,
    pub puuid: String,
    pub game_name: String,
    pub tag_line: String,
    pub region: String,
    pub user_info: String,
    pub authorized_cookies: String,
    pub session: Option<UserSession>,
}

#[derive(Debug)]
pub struct UserSession {
    pub id: i64,
    pub user_id: i64,
    pub access_token: String,
    pub id_token: String,
    pub expires_at: DateTime<Utc>,
}
