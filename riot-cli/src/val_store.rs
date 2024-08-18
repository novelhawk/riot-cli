use std::{collections::HashMap, ops::Not};

use chrono::{Duration, Utc};
use directories::ProjectDirs;
use val_api::{
    endpoints::{
        self,
        auth::silent_login,
        user::{get_entitlements_token, get_region, get_user_info},
    },
    models::{EmbedImage, MessageEmbed, SkinData, SkinDetails, WebhookMessage},
    thirdparty::{self, discord::send_webhook},
};
use val_login_webview2::{login_popup, RIOT_AUTH_PAGE};

use crate::{
    datastore::Datastore,
    models::{AddUser, AddUserSession, User},
    ValStoreCommands,
};

pub async fn handle_val_store_command(command: &ValStoreCommands) {
    let datastore = Datastore::initialize().expect("database should initialize");

    match command {
        ValStoreCommands::Add => drop(add_account(&datastore).await),
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

    let (not_logged, expired): (Vec<_>, Vec<_>) = users
        .into_iter()
        .filter(|user| {
            user.session
                .as_ref()
                .is_some_and(|s| s.expires_at >= Utc::now())
                .not()
        })
        .partition(|user| user.session.as_ref().is_none());

    let mut failed_refresh = Vec::new();

    for user in expired {
        let cookies = user.session.as_ref().unwrap().authorized_cookies.clone();
        let Some((tokens, cookies)) = silent_login(&cookies).await else {
            failed_refresh.push(user);
            continue;
        };

        db.set_session(&AddUserSession {
            user_id: user.id,
            access_token: tokens.access_token,
            id_token: tokens.id_token,
            expires_at: Utc::now() + Duration::seconds(tokens.expires_in as i64),
            authorized_cookies: cookies,
        })
        .expect("session should be updated")
    }

    for user in not_logged.into_iter().chain(failed_refresh) {
        println!("Login for {}#{}", user.game_name, user.tag_line);
        let user_id = add_account(&db).await;
        if user_id != user.id {
            eprintln!("Logged in on the wrong account!");
        }
    }
}

pub async fn add_account(db: &Datastore) -> i64 {
    let project = ProjectDirs::from("", "", "riot-cli").expect("project dirs should work");
    let folder = project.data_dir().join("edge-profile");

    let (tokens, cookies) = login_popup(&folder, RIOT_AUTH_PAGE).expect("failed to login");
    let (raw_user_info, user_info) = get_user_info(&tokens.access_token).await;
    let region = get_region(&tokens.access_token, tokens.id_token.clone()).await;
    let entitlements_token = get_entitlements_token(&tokens.access_token).await;

    let user = AddUser {
        puuid: user_info.sub.clone(),
        game_name: user_info.acct.game_name.clone(),
        tag_line: user_info.acct.tag_line.clone(),
        region: region.affinities.live,
        user_info: raw_user_info,
        entitlements_token,
        next_nightmarket: Utc::now(),
        next_store: Utc::now(),
    };

    let user_id = db.add_user(&user).expect("user should be added");

    let session = AddUserSession {
        user_id,
        access_token: tokens.access_token.clone(),
        id_token: tokens.id_token.clone(),
        expires_at: Utc::now() + Duration::seconds(tokens.expires_in as i64),
        authorized_cookies: cookies,
    };

    db.set_session(&session).expect("session should be saved");

    println!(
        "Added account {}#{} to database",
        user.game_name, user.tag_line
    );

    user_id
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

        if user.next_store > Utc::now() && !force && !force_nightmarket {
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

                let skins: Vec<_> = bonus
                    .bonus_store_offers
                    .iter()
                    .map(|bonus_offer| SkinData {
                        offer: bonus_offer.offer.clone(),
                        detail: hash.get(&bonus_offer.offer.offer_id).unwrap().clone(),
                        bonus_offer: Some(bonus_offer.clone()),
                    })
                    .collect();

                let message = generate_store_messages(&user, skins);
                send_webhooks(&webhooks, message).await;

                println!("Sent nightmarket of {}#{}", user.game_name, user.tag_line);
            }
        } else if *force_nightmarket {
            println!("No nightmarket available");
        }

        if user.next_store <= Utc::now() || *force {
            let duration = Duration::seconds(
                store
                    .skins_panel_layout
                    .single_item_offers_remaining_duration_in_seconds,
            );
            let next_store = Utc::now() + duration;
            db.set_user_next_store(&user.id, &next_store)
                .expect("failed update next_store");

            let skins: Vec<_> = store
                .skins_panel_layout
                .single_item_store_offers
                .iter()
                .map(|offer| SkinData {
                    offer: offer.clone(),
                    detail: hash.get(&offer.offer_id).unwrap().clone(),
                    bonus_offer: None,
                })
                .collect();

            let message = generate_store_messages(&user, skins);
            send_webhooks(&webhooks, message).await;

            println!("Sent store of user {}#{}", user.game_name, user.tag_line);
        }
    }
}

pub fn generate_store_messages(user: &User, skins: Vec<SkinData>) -> WebhookMessage {
    WebhookMessage {
        username: Some(format!("{}#{}", user.game_name, user.tag_line)),
        content: None,
        embeds: Some(
            skins
                .into_iter()
                .map(|skin| MessageEmbed {
                    title: skin.detail.display_name,
                    description: match &skin.bonus_offer {
                        Some(bonus_offer) => Some(format!(
                            "<:vp:1274118602001350757> ~~{}~~ {} (-{}%)",
                            skin.offer.cost.valorant_points,
                            bonus_offer.discount_costs.valorant_points,
                            bonus_offer.discount_percent,
                        )),
                        None => Some(format!(
                            "<:vp:1274118602001350757> {}",
                            skin.offer.cost.valorant_points,
                        )),
                    },
                    image: None,
                    thumbnail: skin
                        .detail
                        .display_icon
                        .map(|url| EmbedImage { url })
                        .or(skin.detail.levels[0]
                            .clone()
                            .display_icon
                            .map(|url| EmbedImage { url }))
                        .clone(),
                    color: Some(if skin.bonus_offer.as_ref().is_none() {
                        0x6cc551
                    } else {
                        0xff00aa
                    }),
                    timestamp: None,
                    footer: None,
                })
                .collect(),
        ),
    }
}

pub async fn send_webhooks(webhooks: &Vec<String>, message: WebhookMessage) {
    for webhook in webhooks {
        send_webhook(webhook, &message).await;
    }
}
