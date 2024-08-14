pub mod endpoints;
pub mod models;
pub mod unofficial;

use std::{
    collections::{BTreeMap, HashMap},
    io::{BufRead, BufReader, BufWriter},
    sync::Arc,
};

use reqwest::{
    header::{self, HeaderValue},
    redirect::Policy,
    StatusCode,
};
use reqwest_cookie_store::CookieStoreMutex;
use rustls::{
    cipher_suite::{
        TLS13_AES_128_GCM_SHA256, TLS13_AES_256_GCM_SHA384, TLS13_CHACHA20_POLY1305_SHA256,
    },
    version::TLS13,
    RootCertStore, ALL_KX_GROUPS,
};

use crate::val_api::models::{EntitlementsTokenResponse, RegionRequest, RegionResponse};

use self::models::{
    AuthClientRequest, AuthResult, LoginRequest, LoginResponse, Tokens, UserInfoResponse,
};

const USER_AGENT: HeaderValue = HeaderValue::from_static(
    "RiotClient/51.0.0.4429735.4381201 rso-auth (Windows;10;;Professional, x64)",
);

const CONTENT_TYPE: HeaderValue = HeaderValue::from_static("application/json");

const ACCEPT: HeaderValue = HeaderValue::from_static("application/json, text/plain, */*");

pub async fn authenticate(username: String, password: String) -> AuthResult {
    // Create cookie store
    let cookies = reqwest_cookie_store::CookieStore::default();
    let cookies = reqwest_cookie_store::CookieStoreMutex::new(cookies);
    let cookies = Arc::new(cookies);

    // Prepare headers -- Order matters!
    let mut headers = header::HeaderMap::new();
    headers.insert(header::HOST, HeaderValue::from_static("auth.riotgames.com"));
    headers.insert(header::CONTENT_TYPE, CONTENT_TYPE);
    headers.insert(header::ACCEPT, ACCEPT);
    headers.insert(header::USER_AGENT, USER_AGENT);
    headers.insert(
        header::ACCEPT_ENCODING,
        HeaderValue::from_static("gzip, deflate"),
    );

    // Prepare TLS client
    let mut cert_store = RootCertStore::empty();
    cert_store.add_server_trust_anchors(webpki_roots::TLS_SERVER_ROOTS.0.iter().map(|ta| {
        rustls::OwnedTrustAnchor::from_subject_spki_name_constraints(
            ta.subject,
            ta.spki,
            ta.name_constraints,
        )
    }));

    let tls = rustls::ClientConfig::builder()
        .with_cipher_suites(&[
            TLS13_AES_256_GCM_SHA384,
            TLS13_CHACHA20_POLY1305_SHA256,
            TLS13_AES_128_GCM_SHA256,
        ])
        .with_kx_groups(&ALL_KX_GROUPS)
        .with_protocol_versions(&[&TLS13])
        .unwrap()
        .with_root_certificates(cert_store)
        .with_no_client_auth();

    // Create reqwest client
    let client = reqwest::ClientBuilder::new()
        .cookie_store(true)
        .cookie_provider(cookies.clone())
        .default_headers(headers)
        .use_preconfigured_tls(tls)
        .build()
        .unwrap();

    // Create login cookies
    const ENDPOINT: &str = "https://auth.riotgames.com/api/v1/authorization";

    let req = AuthClientRequest {
        client_id: "play-valorant-web-prod".to_string(),
        nonce: "1".to_string(),
        redirect_uri: "https://playvalorant.com/opt_in".to_string(),
        response_type: "token id_token".to_string(),
        scope: "account openid".to_string(),
    };

    client.post(ENDPOINT).json(&req).send().await.unwrap();

    // Login
    let req = LoginRequest {
        action_type: "auth".to_string(),
        username,
        password,
        remember: true,
    };

    let res = client.put(ENDPOINT).json(&req).send().await.unwrap();

    let text = res.text().await.unwrap();

    let res = serde_json::from_str::<LoginResponse>(text.as_str()).unwrap();

    // Handle result
    if res.response_type != "response" {
        panic!("Response type {} is not supported", res.response_type);
    }

    let url = url::Url::parse(res.response.parameters.uri.as_str()).unwrap();
    let data = url.fragment().unwrap();

    let tokens: Tokens = serde_urlencoded::from_str(data).unwrap();

    let mut writer = BufWriter::new(Vec::new());
    {
        let store = cookies.lock().unwrap();
        store.save_json(&mut writer).unwrap();
    }
    let cookies = String::from_utf8_lossy(writer.buffer());

    AuthResult {
        tokens,
        time: chrono::Utc::now(),
        cookies: cookies.to_string(),
    }
}

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

pub async fn get_region(access_token: String, id_token: String) -> RegionResponse {
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

pub async fn silent_login(cookies: &String) -> Option<(Tokens, String)> {
    let json = BufReader::new(cookies.as_bytes());
    let cookie_store = reqwest_cookie_store::CookieStore::load_json(json).unwrap();
    let cookie_store = reqwest_cookie_store::CookieStoreMutex::new(cookie_store);
    let cookie_store = std::sync::Arc::new(cookie_store);

    const ENDPOINT: &str = concat!(
        "https://auth.riotgames.com/authorize?",
        "redirect_uri=https%3A%2F%2Fplayvalorant.com%2Fopt_in&",
        "client_id=play-valorant-web-prod&",
        "response_type=token%20id_token&",
        "scope=account%20openid&",
        "nonce=1"
    );

    // Prepare TLS client
    let mut cert_store = RootCertStore::empty();
    cert_store.add_server_trust_anchors(webpki_roots::TLS_SERVER_ROOTS.0.iter().map(|ta| {
        rustls::OwnedTrustAnchor::from_subject_spki_name_constraints(
            ta.subject,
            ta.spki,
            ta.name_constraints,
        )
    }));

    let tls = rustls::ClientConfig::builder()
        .with_cipher_suites(&[
            TLS13_AES_256_GCM_SHA384,
            TLS13_CHACHA20_POLY1305_SHA256,
            TLS13_AES_128_GCM_SHA256,
        ])
        .with_kx_groups(&ALL_KX_GROUPS)
        .with_protocol_versions(&[&TLS13])
        .unwrap()
        .with_root_certificates(cert_store)
        .with_no_client_auth();

    let client = reqwest::ClientBuilder::new()
        .cookie_store(true)
        .cookie_provider(cookie_store.clone())
        .redirect(Policy::none())
        .use_preconfigured_tls(tls)
        .build()
        .unwrap();

    let res = client.get(ENDPOINT).send().await.unwrap();

    let status = res.status();
    println!("Status: {status:?}");

    if let Some(location) = res.headers().get(header::LOCATION) {
        let url = url::Url::parse(location.to_str().unwrap()).unwrap();
        if url.path().starts_with("/login") {
            None
        } else {
            let fragment = url.fragment().unwrap();
            let tokens = serde_urlencoded::from_str::<Tokens>(fragment).unwrap();

            let mut writer = BufWriter::new(Vec::new());
            {
                let store = cookie_store.lock().unwrap();
                store.save_json(&mut writer).unwrap();
            }
            let cookies = String::from_utf8_lossy(writer.buffer());

            Some((tokens, cookies.to_string()))
        }
    } else {
        None
    }
}
