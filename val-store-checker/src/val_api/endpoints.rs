use super::models::StoreFrontResponse;

const PLATFORM: &str = "ew0KCSJwbGF0Zm9ybVR5cGUiOiAiUEMiLA0KCSJwbGF0Zm9ybU9TIjogIldpbmRvd3MiLA0KCSJwbGF0Zm9ybU9TVmVyc2lvbiI6ICIxMC4wLjE5MDQyLjEuMjU2LjY0Yml0IiwNCgkicGxhdGZvcm1DaGlwc2V0IjogIlVua25vd24iDQp9";

pub async fn store_fetch_offers(
    client_ver: String,
    ent: String,
    access_token: String,
    shard: String,
) {
    let url = format!("https://pd.{shard}.a.pvp.net/store/v1/offers/");

    let client = reqwest::Client::new();

    let res = client
        .get(url)
        .header("X-Riot-ClientPlatform", PLATFORM)
        .header("X-Riot-ClientVersion", client_ver)
        .header("X-Riot-Entitlements-JWT", ent)
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await
        .unwrap();

    println!("{}", res.text().await.unwrap());
}

pub async fn store_fetch_storefront(
    client_ver: String,
    ent: String,
    access_token: String,
    shard: String,
    puuid: String,
) -> StoreFrontResponse {
    let url = format!("https://pd.{shard}.a.pvp.net/store/v2/storefront/{puuid}");

    let client = reqwest::Client::new();

    let res = client
        .get(url)
        .header("X-Riot-ClientPlatform", PLATFORM)
        .header("X-Riot-ClientVersion", client_ver)
        .header("X-Riot-Entitlements-JWT", ent)
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await
        .unwrap();

    res.json::<StoreFrontResponse>().await.unwrap()
}
