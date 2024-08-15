use reqwest::header::{self, HeaderValue};

use crate::models::{EntitlementsTokenResponse, RegionRequest, RegionResponse, UserInfoResponse};

const CONTENT_TYPE: HeaderValue = HeaderValue::from_static("application/json");

pub async fn get_entitlements_token(access_token: &str) -> String {
    let mut headers = header::HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, CONTENT_TYPE);
    headers.insert(
        header::AUTHORIZATION,
        HeaderValue::from_str(format!("Bearer {}", access_token).as_str()).unwrap(),
    );

    let client = reqwest::Client::new();

    const ENDPOINT: &str = "https://entitlements.auth.riotgames.com/api/token/v1";

    let res = client.post(ENDPOINT).headers(headers).send().await.unwrap();

    let res = res.json::<EntitlementsTokenResponse>().await.unwrap();

    res.entitlements_token
}

pub async fn get_user_info(access_token: &str) -> (String, UserInfoResponse) {
    let mut headers = header::HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, CONTENT_TYPE);
    headers.insert(
        header::AUTHORIZATION,
        HeaderValue::from_str(format!("Bearer {}", access_token).as_str()).unwrap(),
    );

    let client = reqwest::Client::new();

    const ENDPOINT: &str = "https://auth.riotgames.com/userinfo";

    let res = client.post(ENDPOINT).headers(headers).send().await.unwrap();
    let text = res.text().await.unwrap();

    let user_info = serde_json::from_str::<UserInfoResponse>(text.as_str()).unwrap();

    (text, user_info)
}

pub async fn get_region(access_token: &String, id_token: String) -> RegionResponse {
    let mut headers = header::HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, CONTENT_TYPE);
    headers.insert(
        header::AUTHORIZATION,
        HeaderValue::from_str(format!("Bearer {}", access_token).as_str()).unwrap(),
    );

    let client = reqwest::Client::new();

    const ENDPOINT: &str = "https://riot-geo.pas.si.riotgames.com/pas/v1/product/valorant";
    let res = client
        .put(ENDPOINT)
        .json(&RegionRequest { id_token })
        .headers(headers)
        .send()
        .await
        .unwrap();

    res.json::<RegionResponse>().await.unwrap()
}
