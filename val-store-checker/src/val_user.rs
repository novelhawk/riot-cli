use std::collections::BTreeMap;

use chrono::{DateTime, Duration, Utc};
use val_login_webview::tokens::Tokens;
// use surrealdb::{sql::{Value, Strand, Datetime}, Datastore, Session};

use crate::val_api::models::RegionResponse;

#[derive(Debug)]
pub struct User {
    pub username: String,
    pub access_token: String,
    pub id_token: String,
    pub expires: DateTime<Utc>,
    pub entitlements_token: Option<String>,
    pub puuid: Option<String>,
    pub region: Option<String>,
    pub game_name: Option<String>,
    pub tag_line: Option<String>,
    pub authorized_cookies: Option<String>,
    pub next_store: Option<DateTime<Utc>>,

    pub user_info: Option<String>,
    pub region_info: Option<RegionResponse>,
}

impl User {
    // pub async fn save(&self, ds: &Datastore, sess: &Session) {
    //     let mut data = BTreeMap::new();

    //     data.insert("username".to_string(), Strand(self.username.clone()).into());
    //     data.insert("access_token".to_string(), Strand(self.access_token.clone()).into());
    //     data.insert("id_token".to_string(), Strand(self.id_token.clone()).into());
    //     data.insert("expires".to_string(), Datetime(self.expires.clone()).into());
    //     if let Some(user_info) = &self.user_info {
    //         data.insert("user_info".to_string(), Strand(user_info.clone()).into());
    //     }
    //     if let Some(region_info) = &self.region_info {
    //         data.insert("region_info".to_string(), Strand(serde_json::to_string(&region_info).unwrap()).into());
    //     }
    //     if let Some(entitlements_token) = &self.entitlements_token {
    //         data.insert("entitlements_token".to_string(), Strand(entitlements_token.clone()).into());
    //     }
    //     if let Some(puuid) = &self.puuid {
    //         data.insert("puuid".to_string(), Strand(puuid.clone()).into());
    //     }
    //     if let Some(region) = &self.region {
    //         data.insert("region".to_string(), Strand(region.clone()).into());
    //     }
    //     if let Some(game_name) = &self.game_name {
    //         data.insert("game_name".to_string(), Strand(game_name.clone()).into());
    //     }
    //     if let Some(tag_line) = &self.tag_line {
    //         data.insert("tag_line".to_string(), Strand(tag_line.clone()).into());
    //     }
    //     if let Some(authorized_cookies) = &self.authorized_cookies {
    //         data.insert("cookies".to_string(), Strand(authorized_cookies.clone()).into());
    //     }
    //     if let Some(next_store) = &self.next_store {
    //         data.insert("next_store".to_string(), Datetime(next_store.clone()).into());
    //     }

    //     let ast = "UPDATE type::thing('user', $username) CONTENT {
    //         access_token: $access_token,
    //         id_token: $id_token,
    //         expires: $expires,
    //         entitlements_token: $entitlements_token,
    //         cookies: $cookies,
    //         user_info: $user_info,
    //         game_name: $game_name,
    //         tag_line: $tag_line,
    //         puuid: $puuid,
    //         region_info: $region_info,
    //         region: $region,
    //         next_store: $next_store,
    //     }";

    //     ds.execute(ast, &sess, Some(data), false).await.unwrap();
    // }

    pub fn set_tokens(&mut self, tokens: Tokens) {
        self.access_token = tokens.access_token;
        self.id_token = tokens.id_token;
        self.expires = Utc::now() + Duration::seconds(tokens.expires_in as i64);
    }

    pub fn set_next_store(&mut self, remaining_seconds: i64) {
        self.next_store = Some(Utc::now() + Duration::seconds(remaining_seconds));
    }
}

// impl From<Value> for User {
//     fn from(val: Value) -> Self {
//         let data = match val {
//             Value::Object(obj) => obj.0,
//             _ => panic!("Expected object"),
//         };

//         let access_token = match data.get("access_token") {
//             Some(Value::Strand(s)) => s.0.clone(),
//             _ => unreachable!()
//         };

//         let username = match data.get("id") {
//             Some(Value::Thing(s)) => match &s.id {
//                 surrealdb::sql::Id::String(s) => s.clone(),
//                 _ => unreachable!()
//             },
//             _ => unreachable!()
//         };

//         let expires = match data.get("expires") {
//             Some(Value::Datetime(d)) => d.0,
//             _ => unreachable!()
//         };

//         let entitlements_token = match data.get("entitlements_token") {
//             Some(Value::Strand(s)) => Some(s.0.clone()),
//             _ => None
//         };

//         let id_token = match data.get("id_token") {
//             Some(Value::Strand(s)) => s.0.clone(),
//             _ => unreachable!()
//         };

//         let puuid = match data.get("puuid") {
//             Some(Value::Strand(s)) => Some(s.0.clone()),
//             _ => None
//         };

//         let region = match data.get("region") {
//             Some(Value::Strand(s)) => Some(s.0.clone()),
//             _ => None
//         };

//         let game_name = match data.get("game_name") {
//             Some(Value::Strand(s)) => Some(s.0.clone()),
//             _ => None
//         };

//         let user_info = match data.get("user_info") {
//             Some(Value::Strand(s)) => Some(s.0.clone()),
//             _ => None
//         };

//         let region_info = match data.get("region_info") {
//             Some(Value::Strand(s)) => Some(serde_json::from_str(s.0.as_str()).unwrap()),
//             _ => None
//         };

//         let tag_line = match data.get("tag_line") {
//             Some(Value::Strand(s)) => Some(s.0.clone()),
//             _ => None
//         };

//         let cookies = match data.get("cookies") {
//             Some(Value::Strand(s)) => Some(s.0.clone()),
//             _ => None
//         };

//         let next_store = match data.get("next_store") {
//             Some(Value::Datetime(d)) => Some(d.0),
//             _ => None
//         };

//         User {
//             access_token,
//             authorized_cookies: cookies,
//             entitlements_token,
//             game_name,
//             expires,
//             username,
//             user_info,
//             region_info,
//             id_token,
//             puuid,
//             region,
//             tag_line,
//             next_store,
//         }
//     }
// }
