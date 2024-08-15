use std::{
    fs, i64,
    path::{Path, PathBuf},
};

use directories::ProjectDirs;
use rusqlite::{params, Connection, Result};

use crate::models::{User, UserSession};

pub struct Datastore {
    conn: Connection,
}

impl Datastore {
    pub fn initialize() -> Result<Self> {
        let conn = Connection::open(Datastore::get_database_path())?;
        let store = Self { conn };
        store.create_tables()?;

        Ok(store)
    }

    fn get_database_path() -> PathBuf {
        let project = ProjectDirs::from("", "", "riot-cli").expect("project dirs should work");
        let data_dir = project.data_dir();
        fs::create_dir_all(data_dir).expect("data dir should create");
        return data_dir.join("datastore.db");
    }

    pub fn set_session(&self, user_id: i64, session: &UserSession) -> Result<()> {
        self.conn.execute(
            "INSERT INTO sessions (user_id, token, id_token, expires_at)
            VALUES (?1, ?2, ?3, ?4)
            ON CONFLICT (user_id)
            DO
                UPDATE
                SET token = ?2, id_token = ?3, expires_at = ?4",
            params![
                user_id,
                &session.access_token,
                &session.id_token,
                session.expires_at.timestamp_nanos_opt().unwrap_or(i64::MAX),
            ],
        )?;
        Ok(())
    }

    pub fn add_user(&self, user: &User) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO users (puuid, game_name, tag_line, region, user_info, authenticated_cookies)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            [&user.puuid, &user.game_name, &user.tag_line, &user.region, &user.user_info, &user.authorized_cookies]
        )?;

        let user_id = self.conn.last_insert_rowid();

        if let Some(session) = &user.session {
            self.set_session(user_id, session)?;
        }

        Ok(user_id)
    }

    fn create_tables(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY,
                puuid TEXT NOT NULL,
                game_name TEXT NOT NULL,
                tag_line TEXT NOT NULL,
                region TEXT NOT NULL,
                user_info TEXT NOT NULL,
                authenticated_cookies TEXT NOT NULL
            )",
            (),
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS sessions (
                id INTEGER PRIMARY KEY,
                user_id INTEGER UNIQUE NOT NULL,
                token TEXT NOT NULL,
                id_token TEXT NOT NULL,
                expires_at INTEGER NOT NULL,

                FOREIGN KEY (user_id) REFERENCES users (id)
            )",
            (),
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS webhooks (
                id INTEGER PRIMARY KEY,
                url TEXT NOT NULL
            )",
            (),
        )?;

        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS version (
                version TEXT NOT NULL
            )",
            (),
        )?;

        self.conn.execute_batch(
            "DELETE FROM version;
            INSERT INTO version VALUES ('1.0.0');",
        )?;

        Ok(())
    }
}
