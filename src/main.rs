use std::{
    collections::{HashMap},
};

use chrono::{Utc, Local, DateTime};
use surrealdb::{
    sql::{Value},
    Datastore, Response, Session,
};
use val_api::{silent_login, unofficial};
use val_user::User;

use crate::val_api::models::SkinDetails;
use crate::val_api::{
    endpoints,
    models::{EmbedFooter, EmbedImage, MessageEmbed, WebhookMessage},
};

mod val_api;
mod val_user;

#[tokio::main]
async fn main() {
    let ds = Datastore::new("file://data.db").await.unwrap();
    let sess = Session::for_db("val", "val");

    let hash = unofficial::get_weapon_skins().await;
    let hash: HashMap<String, SkinDetails> = hash
        .into_iter()
        .filter(|detail| detail.levels.len() > 0)
        .map(|detail| (detail.levels[0].uuid.clone(), detail))
        .collect();
    let client_ver = unofficial::get_client_version().await;

    let ast = "SELECT * FROM user";
    let res: Vec<Response> = ds.execute(ast, &sess, None, false).await.unwrap();
    let res = res.into_iter().nth(0).unwrap();
    let users = match res.result.unwrap() {
        Value::Array(arr) => arr.0,
        _ => panic!("Expected array"),
    };

    for user in users {
        let mut user: User = user.into();

        match user.next_store {
            Some(next_store) if next_store > Utc::now() => {
                println!(
                    "Next store of {}#{} will be available at {}",
                    user.game_name.clone().unwrap(),
                    user.tag_line.clone().unwrap(),
                    DateTime::<Local>::from(next_store)
                );
                continue;
            }
            _ => {}
        }

        if user.expires <= Utc::now() {
            println!("Expired tokens for {}", user.username);
            match &user.authorized_cookies {
                Some(cookies) => {
                    if let Some((tokens, cookies)) = silent_login(cookies).await {
                        println!("Silent login successful for {}", user.username);
                        user.set_tokens(tokens);
                        user.authorized_cookies = Some(cookies);
                    } else {
                        println!("Cookies are expired for {}", user.username);
                        continue;
                    }
                }
                None => {
                    println!("Session expired for {}", user.username);
                    continue;
                }
            }
        };

        let store = endpoints::store_fetch_storefront(
            client_ver.clone(),
            user.entitlements_token.clone().unwrap(),
            user.access_token.clone(),
            user.region.clone().unwrap(),
            user.puuid.clone().unwrap(),
        )
        .await;

        user.set_next_store(
            store
                .skins_panel_layout
                .single_item_offers_remaining_duration_in_seconds,
        );

        let skins: Vec<SkinDetails> = store
            .bonus_store.unwrap()
            .bonus_store_offers
            .iter()
            .map(|offer| hash.get(&offer.offer.offer_id).unwrap().clone())
            .collect();

        // let skins: Vec<SkinDetails> = store
        //     .skins_panel_layout
        //     .single_item_offers
        //     .iter()
        //     .map(|skin| hash.get(skin).unwrap().clone())
        //     .collect();

        webhook(
            format!(
                "{}#{}",
                user.game_name.clone().unwrap(),
                user.tag_line.clone().unwrap()
            ),
            skins,
        )
        .await;

        user.save(&ds, &sess).await;
    }
}

async fn webhook(user: String, skins: Vec<SkinDetails>) {
    const WEBHOOK: &str = "https://discord.com/api/webhooks/[REDACTED]";

    let client = reqwest::Client::new();

    let message = WebhookMessage {
        username: Some(user),
        content: None,
        embeds: Some(
            skins
                .into_iter()
                .map(|skin| MessageEmbed {
                    title: skin.display_name,
                    description: None,
                    image: skin
                        .display_icon
                        .map(|url| EmbedImage { url })
                        .or(skin.levels[0].clone().display_icon.map(|url| EmbedImage { url }))
                        .clone(),
                    color: Some(0xff00aa),
                    timestamp: Some(chrono::Utc::now()),
                    footer: Some(EmbedFooter {
                        text: Some("Automatic Valorant Shop Checker".to_string()),
                        icon_url: None,
                        proxy_icon_url: None,
                    }),
                })
                .collect(),
        ),
    };

    client.post(WEBHOOK).json(&message).send().await.unwrap();
}
