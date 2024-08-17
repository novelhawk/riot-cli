use std::{fs, i64, path::PathBuf};

use chrono::{DateTime, Utc};
use directories::ProjectDirs;
use rusqlite::{params, Connection, Result};

use crate::models::{AddUser, AddUserSession, User, UserSession};

pub struct Datastore {
    conn: Connection,
}

impl Datastore {
    pub fn initialize() -> Result<Self> {
        let conn = Connection::open(Datastore::get_database_path())?;
        let store = Self { conn };
        store.migrate()?;

        Ok(store)
    }

    fn get_database_path() -> PathBuf {
        let project = ProjectDirs::from("", "", "riot-cli").expect("project dirs should work");
        let data_dir = project.data_dir();
        fs::create_dir_all(data_dir).expect("data dir should create");
        return data_dir.join("datastore.db");
    }

    pub fn set_session(&self, session: &AddUserSession) -> Result<()> {
        self.conn.execute(
            "INSERT INTO sessions (user_id, access_token, id_token, expires_at, authenticated_cookies)
            VALUES (?1, ?2, ?3, ?4, ?5)
            ON CONFLICT (user_id)
            DO
                UPDATE
                SET access_token = ?2, id_token = ?3, expires_at = ?4, authenticated_cookies = ?5",
            params![
                &session.user_id,
                &session.access_token,
                &session.id_token,
                session.expires_at.timestamp_nanos_opt().unwrap_or(i64::MAX),
                &session.authorized_cookies,
            ],
        ).map(|_| ())
    }

    pub fn add_user(&self, user: &AddUser) -> Result<i64> {
        self.conn.query_row(
            "INSERT INTO users (puuid, game_name, tag_line, region, user_info, entitlements_token, next_store, next_nightmarket)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            ON CONFLICT (puuid)
            DO
                UPDATE
                SET game_name = ?2, tag_line = ?3, region = ?4, user_info = ?5, entitlements_token = ?6, next_store = ?7, next_nightmarket = ?8
            RETURNING id",
            params![
                &user.puuid,
                &user.game_name,
                &user.tag_line,
                &user.region,
                &user.user_info,
                &user.entitlements_token,
                &user.next_store.timestamp_nanos_opt().unwrap_or(i64::MAX),
                &user.next_nightmarket.timestamp_nanos_opt().unwrap_or(i64::MAX),
            ],
            |row| row.get(0),
        )
    }

    pub fn set_user_next_store(&self, user_id: &i64, next_store: &DateTime<Utc>) -> Result<()> {
        self.conn.execute(
            "UPDATE users
            SET next_store = ?2
            WHERE id = ?1",
            params![
                &user_id,
                &next_store.timestamp_nanos_opt().unwrap_or(i64::MAX),
            ],
        )?;

        Ok(())
    }

    pub fn set_user_next_nightmarket(
        &self,
        user_id: &i64,
        next_nightmarket: &DateTime<Utc>,
    ) -> Result<()> {
        self.conn.execute(
            "UPDATE users
            SET next_nightmarket = ?2
            WHERE id = ?1",
            params![
                &user_id,
                &next_nightmarket.timestamp_nanos_opt().unwrap_or(i64::MAX),
            ],
        )?;

        Ok(())
    }

    pub fn add_webhook(&self, url: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO discord_webhooks (url)
            VALUES (?1)",
            [&url],
        )?;

        Ok(())
    }

    pub fn get_webhooks(&self) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT url
            FROM discord_webhooks",
        )?;

        let urls = stmt.query_map([], |row| row.get(0))?;

        urls.collect()
    }

    pub fn get_users(&self) -> Result<Vec<User>> {
        let mut stmt = self.conn.prepare(
            "SELECT u.id, u.puuid, u.game_name, u.tag_line, u.region, u.user_info, u.entitlements_token, u.next_store, u.next_nightmarket, s.id as session_id, s.access_token, s.id_token, s.expires_at, s.authenticated_cookies
            FROM users u
            LEFT JOIN sessions s ON s.user_id = u.id")?;

        let users = stmt.query_map([], |row| {
            Ok(User {
                id: row.get(0)?,
                puuid: row.get(1)?,
                game_name: row.get(2)?,
                tag_line: row.get(3)?,
                region: row.get(4)?,
                user_info: row.get(5)?,
                entitlements_token: row.get(6)?,
                next_store: DateTime::from_timestamp_nanos(row.get(7)?),
                next_nightmarket: DateTime::from_timestamp_nanos(row.get(8)?),
                session: match row.get(9)? {
                    Some(id) => Some(UserSession {
                        id,
                        user_id: row.get(0)?,
                        access_token: row.get(10)?,
                        id_token: row.get(11)?,
                        expires_at: DateTime::from_timestamp_nanos(row.get(12)?),
                        authorized_cookies: row.get(13)?,
                    }),
                    None => None,
                },
            })
        })?;

        users.collect()
    }

    fn get_database_version(&self) -> Result<usize> {
        let version: String = self.conn.query_row(
            "SELECT version
            FROM version",
            [],
            |row| row.get(0),
        )?;

        Ok(if version == "1.0.0" {
            1
        } else if let Ok(version) = version.parse() {
            version
        } else {
            0
        })
    }

    fn migrate(&self) -> Result<()> {
        let version = self.get_database_version()?;
        if version < 1 {
            self.conn
                .execute_batch(include_str!("../migrations/01_initialize.sql"))?;
        }

        Ok(())
    }
}
