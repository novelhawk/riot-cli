use std::{
    env,
};

use chrono::{Duration};

use surrealdb::{Surreal, engine::remote::ws::Ws, opt::auth::Root};
use val_api::{authenticate, get_entitlements_token};
use val_user::User;
use crate::val_api::{get_region, get_user_info};

mod val_api;
mod val_user;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        match args.get(0) {
            Some(path) => eprintln!("Usage: {} <username> <password>", path),
            _ => eprintln!("Usage: val-store-login <username> <password>"),
        }
        return;
    }

    // let db = Surreal::new::<Ws>("localhost:8000").await?;

    // // Signin as a namespace, database, or root user
    // db.signin(Root {
    //     username: "root",
    //     password: "root",
    // }).await?;

    // // Select a specific namespace / database
    // db.use_ns("namespace").use_db("database").await?;

    login(args[1].clone(), args[2].clone()).await;
}

async fn login(username: String, password: String) {
    let auth = authenticate(username.clone(), password).await;
    let ent = get_entitlements_token(auth.tokens.access_token.as_str()).await;
    let (raw_info, info) = get_user_info(&auth.tokens.access_token).await;
    let region = get_region(
        auth.tokens.access_token.clone(),
        auth.tokens.id_token.clone(),
    )
    .await;

    let user = User {
        username: username.clone(),
        access_token: auth.tokens.access_token.clone(),
        entitlements_token: Some(ent),
        expires: auth.time + Duration::seconds(auth.tokens.expires_in as i64),
        game_name: Some(info.acct.game_name.clone()),
        tag_line: Some(info.acct.tag_line.clone()),
        id_token: auth.tokens.id_token.clone(),
        puuid: Some(info.sub.clone()),
        region: Some(region.affinities.live.clone()),
        region_info: Some(region),
        user_info: Some(raw_info.clone()),
        authorized_cookies: Some(auth.cookies),
        next_store: None,
    };

    println!("{user:#?}");
}

