use chrono::{Duration, Utc};
use val_api::endpoints::user::{get_region, get_user_info};
use val_login_webview::{login_popup, RIOT_AUTH_PAGE};

use crate::{
    datastore::Datastore,
    models::{User, UserSession},
    ValStoreCommands,
};

pub async fn handle_val_store_command(command: &ValStoreCommands) {
    let datastore = Datastore::initialize().expect("database should initialize");

    match command {
        ValStoreCommands::Add => add_account(&datastore).await,
        ValStoreCommands::Check => check(&datastore).await,
    };
}

pub async fn add_account(db: &Datastore) {
    let tokens = login_popup(RIOT_AUTH_PAGE).expect("failed to login");
    let (raw_user_info, user_info) = get_user_info(&tokens.access_token).await;
    let region = get_region(&tokens.access_token, tokens.id_token.clone()).await;

    let session = UserSession {
        id: 0,
        user_id: 0,
        access_token: tokens.access_token.clone(),
        id_token: tokens.id_token.clone(),
        expires_at: Utc::now() + Duration::seconds(tokens.expires_in as i64),
    };

    let user = User {
        id: 0,
        puuid: user_info.sub.clone(),
        game_name: user_info.acct.game_name.clone(),
        tag_line: user_info.acct.tag_line.clone(),
        region: region.affinities.live,
        user_info: raw_user_info,
        authorized_cookies: String::new(),
        session: Some(session),
    };

    db.add_user(&user).expect("user should be added");

    println!("Added account {user:?} to database");
}

pub async fn check(db: &Datastore) {}
