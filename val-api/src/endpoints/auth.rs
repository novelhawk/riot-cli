use std::sync::Arc;

use reqwest::{
    header::{self},
    redirect::Policy,
};
use rustls::{
    crypto::{
        aws_lc_rs::{
            self,
            cipher_suite::{
                TLS13_AES_128_GCM_SHA256, TLS13_AES_256_GCM_SHA384, TLS13_CHACHA20_POLY1305_SHA256,
            },
        },
        CryptoProvider,
    },
    version::TLS13,
    ClientConfig,
};
use rustls_native_certs::load_native_certs;

use crate::models::Tokens;

pub async fn silent_login(cookies: &String) -> Option<(Tokens, String)> {
    let cookie_store = reqwest_cookie_store::CookieStoreMutex::default();
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
    let root_store = {
        let certs = load_native_certs().expect("system certificates should load");
        let mut root_store = rustls::RootCertStore::empty();
        root_store.add_parsable_certificates(certs);
        root_store
    };

    let crypto_provider = CryptoProvider {
        cipher_suites: vec![
            TLS13_AES_128_GCM_SHA256,
            TLS13_AES_256_GCM_SHA384,
            TLS13_CHACHA20_POLY1305_SHA256,
        ],
        kx_groups: aws_lc_rs::ALL_KX_GROUPS.to_vec(),
        ..aws_lc_rs::default_provider()
    };

    let tls = ClientConfig::builder_with_provider(Arc::new(crypto_provider))
        .with_protocol_versions(&[&TLS13])
        .expect("configuration should be valid")
        .with_root_certificates(root_store)
        .with_no_client_auth();

    let client = reqwest::ClientBuilder::new()
        .cookie_store(true)
        .cookie_provider(cookie_store.clone())
        .redirect(Policy::none())
        .use_preconfigured_tls(tls)
        .build()
        .unwrap();

    let res = client
        .get(ENDPOINT)
        .header(header::COOKIE, cookies)
        .send()
        .await
        .unwrap();

    let location = res
        .headers()
        .get(header::LOCATION)
        .and_then(|loc| loc.to_str().ok());

    let tokens: Tokens = location
        .and_then(|str| url::Url::parse(str).ok())
        .and_then(|url| {
            url.fragment()
                .and_then(|frag| serde_urlencoded::from_str(frag).ok())
        })?;

    let new_cookies = {
        let store = cookie_store.lock().unwrap();
        let cookies =
            store.get_request_values(&"https://auth.riotgames.com/authorize".parse().unwrap());
        cookies
            .map(|it| format!("{}={}", it.0, it.1))
            .collect::<Vec<_>>()
            .join("; ")
    };

    Some((tokens, new_cookies))
}
