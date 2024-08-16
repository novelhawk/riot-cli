use std::collections::HashMap;

use chrono::{Duration, Utc};
use url::Url;
use val_api::{
    endpoints::{
        self,
        auth::silent_login,
        user::{get_entitlements_token, get_region, get_user_info},
    },
    models::{EmbedFooter, EmbedImage, MessageEmbed, SkinDetails, WebhookMessage},
    thirdparty::{self, discord::send_webhook},
};
use val_login_webview2::{login_popup, RIOT_AUTH_PAGE};

use crate::{
    datastore::Datastore,
    models::{User, UserSession},
    ValStoreCommands,
};

pub async fn handle_val_store_command(command: &ValStoreCommands) {
    let datastore = Datastore::initialize().expect("database should initialize");

    match command {
        ValStoreCommands::Add => add_account(&datastore).await,
        ValStoreCommands::Check {
            force,
            force_nightmarket,
        } => check(&datastore, force, force_nightmarket).await,
        ValStoreCommands::Webhook { uri } => webhook(&datastore, uri).await,
    };
}

pub async fn webhook(db: &Datastore, uri: &String) {
    db.add_webhook(&uri).expect("failed to add webhook");
    println!("Added url as a discord webhook");
}

pub async fn refresh_expired_accounts(db: &Datastore) {
    let users = db.get_users().expect("users");

    for user in users {
        if let Some(session) = user.session {
            if session.expires_at < Utc::now() {
                if let Some((tokens, cookies)) = silent_login(&session.authorized_cookies).await {
                    db.set_session(
                        user.id,
                        &UserSession {
                            id: 0,
                            user_id: 0,
                            access_token: tokens.access_token,
                            id_token: tokens.id_token,
                            expires_at: Utc::now() + Duration::seconds(tokens.expires_in as i64),
                            authorized_cookies: cookies,
                        },
                    )
                    .unwrap();
                } else {
                    println!("Login for {}#{}", user.game_name, user.tag_line);
                    add_account(&db).await;
                }
            } else {
                // ok
            }
        } else {
            println!("Login for {}#{}", user.game_name, user.tag_line);
            add_account(&db).await;
        }
    }
}

pub async fn add_account(db: &Datastore) {
    let (tokens, cookies) = login_popup(RIOT_AUTH_PAGE).expect("failed to login");
    let (raw_user_info, user_info) = get_user_info(&tokens.access_token).await;
    let region = get_region(&tokens.access_token, tokens.id_token.clone()).await;
    let entitlements_token = get_entitlements_token(&tokens.access_token).await;

    let session = UserSession {
        id: 0,
        user_id: 0,
        access_token: tokens.access_token.clone(),
        id_token: tokens.id_token.clone(),
        expires_at: Utc::now() + Duration::seconds(tokens.expires_in as i64),
        authorized_cookies: cookies,
    };

    let user = User {
        id: 0,
        puuid: user_info.sub.clone(),
        game_name: user_info.acct.game_name.clone(),
        tag_line: user_info.acct.tag_line.clone(),
        region: region.affinities.live,
        user_info: raw_user_info,
        entitlements_token,
        next_nightmarket: Utc::now(),
        next_store: Utc::now(),
        session: Some(session),
    };

    db.add_user(&user).expect("user should be added");

    println!(
        "Added account {}#{} to database",
        user.game_name, user.tag_line
    );
}

pub async fn check(db: &Datastore, force: &bool, force_nightmarket: &bool) {
    refresh_expired_accounts(&db).await;

    let hash = thirdparty::valdata::get_weapon_skins().await;
    let hash: HashMap<String, SkinDetails> = hash
        .into_iter()
        .filter(|detail| detail.levels.len() > 0)
        .map(|detail| (detail.levels[0].uuid.clone(), detail))
        .collect();
    let client_ver = thirdparty::valdata::get_client_version().await;

    let users = db.get_users().expect("users");
    let webhooks = db.get_webhooks().expect("webhooks");

    for user in users {
        let Some(session) = &user.session else {
            println!("User {}#{} is not logged", user.game_name, user.tag_line);
            continue;
        };

        if user.next_store > Utc::now() && !force {
            println!(
                "Skipping user {}#{}, next shop at {}",
                user.game_name, user.tag_line, user.next_store
            );
            continue;
        }

        let store = endpoints::store::store_fetch_storefront(
            &client_ver,
            &user.entitlements_token,
            &session.access_token,
            &user.region,
            &user.puuid,
        )
        .await;

        if let Some(bonus) = store.bonus_store {
            if user.next_nightmarket <= Utc::now() || *force_nightmarket {
                let duration = Duration::seconds(bonus.bonus_store_remaining_duration_in_seconds);
                let next_store = Utc::now() + duration;
                db.set_user_next_nightmarket(&user.id, &next_store)
                    .expect("failed update next_market");

                let skins: Vec<SkinDetails> = bonus
                    .bonus_store_offers
                    .iter()
                    .map(|offer| hash.get(&offer.offer.offer_id).unwrap().clone())
                    .collect();

                send_webhooks(&user, &webhooks, skins).await;
            }
        }

        let duration = Duration::seconds(
            store
                .skins_panel_layout
                .single_item_offers_remaining_duration_in_seconds,
        );
        let next_store = Utc::now() + duration;
        db.set_user_next_store(&user.id, &next_store)
            .expect("failed update next_store");

        let skins: Vec<SkinDetails> = store
            .skins_panel_layout
            .single_item_offers
            .iter()
            .map(|skin| hash.get(skin).unwrap().clone())
            .collect();

        send_webhooks(&user, &webhooks, skins).await;
    }
}

pub async fn send_webhooks(user: &User, webhooks: &Vec<String>, skins: Vec<SkinDetails>) {
    let message = WebhookMessage {
        username: Some(format!("{}#{}", user.game_name, user.tag_line)),
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
                        .or(skin.levels[0]
                            .clone()
                            .display_icon
                            .map(|url| EmbedImage { url }))
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

    for webhook in webhooks {
        send_webhook(webhook, &message).await;
    }
}
