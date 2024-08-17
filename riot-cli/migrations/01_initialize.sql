CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY,
    puuid TEXT UNIQUE NOT NULL,
    game_name TEXT NOT NULL,
    tag_line TEXT NOT NULL,
    region TEXT NOT NULL,
    user_info TEXT NOT NULL,
    entitlements_token TEXT NOT NULL,
    next_store INTEGER NOT NULL,
    next_nightmarket INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS sessions (
    id INTEGER PRIMARY KEY,
    user_id INTEGER UNIQUE NOT NULL,
    access_token TEXT NOT NULL,
    id_token TEXT NOT NULL,
    expires_at INTEGER NOT NULL,
    authenticated_cookies TEXT NOT NULL,

    FOREIGN KEY (user_id) REFERENCES users (id)
);

CREATE TABLE IF NOT EXISTS discord_webhooks (
    id INTEGER PRIMARY KEY,
    url TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS version (
    version TEXT NOT NULL
);

DELETE FROM version;
INSERT INTO version VALUES ('1.0.0');

